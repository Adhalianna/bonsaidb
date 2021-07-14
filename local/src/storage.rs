use std::{
    any::Any,
    collections::HashMap,
    convert::TryFrom,
    fmt::{Debug, Display},
    marker::PhantomData,
    path::Path,
    sync::Arc,
};

use async_trait::async_trait;
use futures::TryFutureExt;
use itertools::Itertools;
#[cfg(feature = "pubsub")]
pub use pliantdb_core::circulate::Relay;
#[cfg(feature = "internal-apis")]
use pliantdb_core::custodian_password::{LoginResponse, ServerLogin};
#[cfg(feature = "keyvalue")]
use pliantdb_core::kv::{KeyOperation, Output};
use pliantdb_core::{
    admin::{
        database::{self, ByName, Database as DatabaseRecord},
        password_config::PasswordConfig,
        user::{self, User},
        Admin,
    },
    connection::{self, AccessPolicy, Connection, QueryKey, ServerConnection},
    custodian_password::ServerRegistration,
    document::{Document, KeyId},
    networking,
    permissions::Permissions,
    schema::{
        view::map, CollectionDocument, CollectionName, MappedValue, Schema, SchemaName, Schematic,
        ViewName,
    },
    transaction::{Executed, OperationResult, Transaction},
};
use pliantdb_jobs::manager::Manager;
use rand::{thread_rng, Rng};
use tokio::{
    fs::{self, File},
    io::{AsyncReadExt, AsyncWriteExt},
    sync::RwLock,
};

use crate::{
    config::Configuration,
    tasks::TaskManager,
    vault::{self, LocalVaultKeyStorage, Vault},
    Database, Error,
};

#[cfg(feature = "keyvalue")]
pub mod kv;

/// A file-based, multi-database, multi-user database engine.
#[derive(Debug, Clone)]
pub struct Storage {
    data: Arc<Data>,
}

#[derive(Debug)]
struct Data {
    id: StorageId,
    sled: sled::Db,
    pub(crate) tasks: TaskManager,
    pub(crate) vault: Arc<Vault>,
    schemas: RwLock<HashMap<SchemaName, Box<dyn DatabaseOpener>>>,
    available_databases: RwLock<HashMap<String, SchemaName>>,
    default_encryption_key: Option<KeyId>,
    pub(crate) check_view_integrity_on_database_open: bool,
    #[cfg(feature = "keyvalue")]
    kv_expirer: std::sync::RwLock<Option<flume::Sender<kv::ExpirationUpdate>>>,
    #[cfg(feature = "pubsub")]
    relay: Relay,
}

impl Storage {
    /// Creates or opens a multi-database [`Storage`] with its data stored in `directory`.
    pub async fn open_local<P: AsRef<Path> + Send>(
        path: P,
        configuration: Configuration,
    ) -> Result<Self, Error> {
        let owned_path = path.as_ref().to_owned();

        let manager = Manager::default();
        for _ in 0..configuration.workers.worker_count {
            manager.spawn_worker();
        }
        let tasks = TaskManager::new(manager);

        fs::create_dir_all(&owned_path)
            .await
            .map_err(|err| Error::Other(Arc::new(anyhow::Error::from(err))))?;

        let id = Self::lookup_or_create_id(&configuration, &owned_path).await?;

        let vault_key_storage = match configuration.vault_key_storage {
            Some(storage) => storage,
            None => Box::new(
                LocalVaultKeyStorage::new(owned_path.join("vault-keys"))
                    .await
                    .map_err(|err| Error::Vault(vault::Error::Initializing(err.to_string())))?,
            ),
        };

        let vault = Arc::new(Vault::initialize(id, &owned_path, vault_key_storage).await?);

        let check_view_integrity_on_database_open = configuration.views.check_integrity_on_open;
        let default_encryption_key = configuration.default_encryption_key;
        let storage = tokio::task::spawn_blocking(move || {
            sled::open(owned_path)
                .map(|sled| Self {
                    data: Arc::new(Data {
                        id,
                        sled,
                        tasks,
                        vault,
                        default_encryption_key,
                        schemas: RwLock::default(),
                        available_databases: RwLock::default(),
                        check_view_integrity_on_database_open,
                        #[cfg(feature = "keyvalue")]
                        kv_expirer: std::sync::RwLock::default(),
                        #[cfg(feature = "pubsub")]
                        relay: Relay::default(),
                    }),
                })
                .map_err(Error::from)
        })
        .await
        .map_err(|err| pliantdb_core::Error::Database(err.to_string()))??;

        #[cfg(feature = "keyvalue")]
        storage
            .data
            .tasks
            .spawn_key_value_expiration_loader(&storage)
            .await;

        storage.cache_available_databases().await?;

        storage.create_admin_database_if_needed().await?;

        Ok(storage)
    }

    async fn lookup_or_create_id(
        configuration: &Configuration,
        path: &Path,
    ) -> Result<StorageId, Error> {
        Ok(StorageId(if let Some(id) = configuration.unique_id {
            // The configuraiton id override is not persisted to disk. This is
            // mostly to prevent someone from accidentally adding this
            // configuration, realizing it breaks things, and then wanting to
            // revert. This makes reverting to the old value easier.
            id
        } else {
            // Load/Store a randomly generated id into a file. While the value
            // is numerical, the file contents are the ascii decimal, making it
            // easier for a human to view, and if needed, edit.
            let id_path = path.join("server-id");

            if id_path.exists() {
                // This value is important enought to not allow launching the
                // server if the file can't be read or contains unexpected data.
                let existing_id = String::from_utf8(
                    File::open(id_path)
                        .and_then(|mut f| async move {
                            let mut bytes = Vec::new();
                            f.read_to_end(&mut bytes).await.map(|_| bytes)
                        })
                        .await
                        .expect("error reading server-id file"),
                )
                .expect("server-id contains invalid data");

                existing_id.parse().expect("server-id isn't numeric")
            } else {
                let id = { thread_rng().gen::<u64>() };
                File::create(id_path)
                    .and_then(|mut file| async move {
                        let id = id.to_string();
                        file.write_all(id.as_bytes()).await?;
                        file.shutdown().await
                    })
                    .await
                    .map_err(|err| {
                        Error::Core(pliantdb_core::Error::Configuration(format!(
                            "Error writing server-id file: {}",
                            err
                        )))
                    })?;
                id
            }
        }))
    }

    async fn cache_available_databases(&self) -> Result<(), Error> {
        let available_databases = self
            .admin()
            .await
            .view::<ByName>()
            .query()
            .await?
            .into_iter()
            .map(|map| (map.key, map.value))
            .collect();
        let mut storage_databases = self.data.available_databases.write().await;
        *storage_databases = available_databases;
        Ok(())
    }

    async fn create_admin_database_if_needed(&self) -> Result<(), Error> {
        self.register_schema::<Admin>().await?;
        match self.database::<Admin>("admin").await {
            Ok(_) => {}
            Err(Error::Core(pliantdb_core::Error::DatabaseNotFound(_))) => {
                self.create_database::<Admin>("admin").await?;
            }
            Err(err) => return Err(err),
        }
        Ok(())
    }

    /// Returns the unique id of the server.
    ///
    /// This value is set from the [`Configuration`] or randomly generated when
    /// creating a server. It shouldn't be changed after a server is in use, as
    /// doing can cause issues. For example, the vault that manages encrypted
    /// storage uses the server ID to store the vault key. If the server ID
    /// changes, the vault key storage will need to be updated with the new
    /// server ID.
    #[must_use]
    pub fn unique_id(&self) -> StorageId {
        self.data.id
    }

    #[must_use]
    pub(crate) fn vault(&self) -> &Vault {
        &self.data.vault
    }

    #[must_use]
    pub(crate) fn default_encryption_key(&self) -> Option<&KeyId> {
        self.data.default_encryption_key.as_ref()
    }

    /// Registers a schema for use within the server.
    pub async fn register_schema<DB: Schema>(&self) -> Result<(), Error> {
        let mut schemas = self.data.schemas.write().await;
        if schemas
            .insert(
                DB::schema_name()?,
                Box::new(StorageSchemaOpener::<DB>::new()?),
            )
            .is_none()
        {
            Ok(())
        } else {
            Err(Error::Core(pliantdb_core::Error::SchemaAlreadyRegistered(
                DB::schema_name()?,
            )))
        }
    }

    /// Retrieves a database. This function checks that the database exists and
    /// that the schema matches.
    pub async fn database<DB: Schema>(&self, name: &'_ str) -> Result<Database<DB>, Error> {
        let available_databases = self.data.available_databases.read().await;

        if let Some(stored_schema) = available_databases.get(name) {
            if stored_schema == &DB::schema_name()? {
                Ok(Database::new(name.to_owned(), self.clone()).await?)
            } else {
                Err(Error::Core(pliantdb_core::Error::SchemaMismatch {
                    database_name: name.to_owned(),
                    schema: DB::schema_name()?,
                    stored_schema: stored_schema.clone(),
                }))
            }
        } else {
            Err(Error::Core(pliantdb_core::Error::DatabaseNotFound(
                name.to_owned(),
            )))
        }
    }

    // pub async fn database_with_foreign_schema<DB: Schema>(
    //     &self,
    //     name: &'_ str,
    // ) -> Result<Database<DB>, Error> {
    //     let available_databases = self.data.available_databases.read().await;

    //     if let Some(stored_schema) = available_databases.get(name) {
    //         Ok(Database::new(name.to_owned(), self.clone()).await?)
    //     } else {
    //         Err(Error::Core(pliantdb_core::Error::DatabaseNotFound(
    //             name.to_owned(),
    //         )))
    //     }
    // }

    pub(crate) fn sled(&self) -> &'_ sled::Db {
        &self.data.sled
    }

    pub(crate) fn tasks(&self) -> &'_ TaskManager {
        &self.data.tasks
    }

    pub(crate) fn check_view_integrity_on_database_open(&self) -> bool {
        self.data.check_view_integrity_on_database_open
    }

    #[cfg(feature = "pubsub")]
    pub(crate) fn relay(&self) -> &'_ Relay {
        &self.data.relay
    }

    #[cfg(feature = "keyvalue")]
    pub(crate) fn update_key_expiration(&self, update: kv::ExpirationUpdate) {
        {
            let sender = self.data.kv_expirer.read().unwrap();
            if let Some(sender) = sender.as_ref() {
                drop(sender.send(update));
                return;
            }
        }

        // If we fall through, we need to initialize the expirer task
        let mut sender = self.data.kv_expirer.write().unwrap();
        if sender.is_none() {
            let (kv_sender, kv_expirer_receiver) = flume::unbounded();
            let thread_sled = self.data.sled.clone();
            tokio::task::spawn_blocking(move || {
                kv::expiration_thread(kv_expirer_receiver, thread_sled)
            });
            *sender = Some(kv_sender);
        }

        drop(sender.as_ref().unwrap().send(update));
    }

    fn validate_name(name: &str) -> Result<(), Error> {
        if name
            .chars()
            .enumerate()
            .all(|(index, c)| c.is_ascii_alphanumeric() || (index > 0 && (c == '.' || c == '-')))
        {
            Ok(())
        } else {
            Err(Error::Core(pliantdb_core::Error::InvalidDatabaseName(
                name.to_owned(),
            )))
        }
    }

    /// Returns the administration database.
    #[allow(clippy::missing_panics_doc)]
    pub async fn admin(&self) -> Database<Admin> {
        Database::new("admin", self.clone()).await.unwrap()
    }

    #[cfg(feature = "internal-apis")]
    #[doc(hidden)]
    /// Opens a database through a generic-free trait.
    pub async fn database_without_schema(
        &self,
        name: &str,
    ) -> Result<Box<dyn OpenDatabase>, Error> {
        let schema = match self
            .admin()
            .await
            .view::<database::ByName>()
            .with_key(name.to_ascii_lowercase())
            .query()
            .await?
            .first()
        {
            Some(entry) => entry.value.clone(),
            None => {
                return Err(Error::Core(pliantdb_core::Error::DatabaseNotFound(
                    name.to_string(),
                )))
            }
        };

        let schemas = self.data.schemas.read().await;
        if let Some(schema) = schemas.get(&schema) {
            schema.open(name.to_string(), self.clone()).await
        } else {
            Err(Error::Core(pliantdb_core::Error::SchemaNotRegistered(
                schema,
            )))
        }
    }

    #[cfg(feature = "internal-apis")]
    #[doc(hidden)]
    /// Authenticates a user.
    pub async fn internal_login_with_password(
        &self,
        username: &str,
        login_request: pliantdb_core::custodian_password::LoginRequest,
    ) -> Result<(Option<u64>, ServerLogin, LoginResponse), pliantdb_core::Error> {
        let admin = self.admin().await;
        let config = PasswordConfig::load(&admin).await?;
        let result = admin
            .view::<user::ByName>()
            .with_key(username.to_owned())
            .query_with_docs()
            .await?;

        let (user_id, existing_password_hash) = if let Some(entry) = result.into_iter().next() {
            let user = entry.document.contents::<User>()?;
            (Some(entry.document.header.id), user.password_hash)
        } else {
            (None, None)
        };

        let (login, response) = ServerLogin::login(&config, existing_password_hash, login_request)?;
        Ok((user_id, login, response))
    }
}

#[async_trait]
trait DatabaseOpener: Send + Sync + Debug {
    fn schematic(&self) -> &'_ Schematic;
    async fn open(&self, name: String, storage: Storage) -> Result<Box<dyn OpenDatabase>, Error>;
}

#[derive(Debug)]
struct StorageSchemaOpener<DB: Schema> {
    schematic: Schematic,
    _phantom: PhantomData<DB>,
}

impl<DB> StorageSchemaOpener<DB>
where
    DB: Schema,
{
    fn new() -> Result<Self, Error> {
        let schematic = DB::schematic()?;
        Ok(Self {
            schematic,
            _phantom: PhantomData::default(),
        })
    }
}

#[async_trait]
impl<DB> DatabaseOpener for StorageSchemaOpener<DB>
where
    DB: Schema,
{
    fn schematic(&self) -> &'_ Schematic {
        &self.schematic
    }

    async fn open(&self, name: String, storage: Storage) -> Result<Box<dyn OpenDatabase>, Error> {
        let db = Database::<DB>::new(name, storage).await?;
        Ok(Box::new(db))
    }
}

/// Methods for accessing a database without the `Schema` type present.
#[doc(hidden)]
#[async_trait]
pub trait OpenDatabase: Send + Sync + Debug + 'static {
    fn as_any(&self) -> &'_ dyn Any;

    async fn get_from_collection_id(
        &self,
        id: u64,
        collection: &CollectionName,
        permissions: &Permissions,
    ) -> Result<Option<Document<'static>>, pliantdb_core::Error>;

    async fn get_multiple_from_collection_id(
        &self,
        ids: &[u64],
        collection: &CollectionName,
        permissions: &Permissions,
    ) -> Result<Vec<Document<'static>>, pliantdb_core::Error>;

    async fn apply_transaction(
        &self,
        transaction: Transaction<'static>,
        permissions: &Permissions,
    ) -> Result<Vec<OperationResult>, pliantdb_core::Error>;

    async fn query(
        &self,
        view: &ViewName,
        key: Option<QueryKey<Vec<u8>>>,
        access_policy: AccessPolicy,
    ) -> Result<Vec<map::Serialized>, pliantdb_core::Error>;

    async fn query_with_docs(
        &self,
        view: &ViewName,
        key: Option<QueryKey<Vec<u8>>>,
        access_policy: AccessPolicy,
        permissions: &Permissions,
    ) -> Result<Vec<networking::MappedDocument>, pliantdb_core::Error>;

    async fn reduce(
        &self,
        view: &ViewName,
        key: Option<QueryKey<Vec<u8>>>,
        access_policy: AccessPolicy,
    ) -> Result<Vec<u8>, pliantdb_core::Error>;

    async fn reduce_grouped(
        &self,
        view: &ViewName,
        key: Option<QueryKey<Vec<u8>>>,
        access_policy: AccessPolicy,
    ) -> Result<Vec<MappedValue<Vec<u8>, Vec<u8>>>, pliantdb_core::Error>;

    async fn list_executed_transactions(
        &self,
        starting_id: Option<u64>,
        result_limit: Option<usize>,
    ) -> Result<Vec<Executed<'static>>, pliantdb_core::Error>;

    async fn last_transaction_id(&self) -> Result<Option<u64>, pliantdb_core::Error>;

    #[cfg(feature = "keyvalue")]
    async fn execute_key_operation(&self, op: KeyOperation)
        -> Result<Output, pliantdb_core::Error>;
}

#[async_trait]
impl ServerConnection for Storage {
    async fn create_database_with_schema(
        &self,
        name: &str,
        schema: SchemaName,
    ) -> Result<(), pliantdb_core::Error> {
        Self::validate_name(name)?;

        {
            let schemas = self.data.schemas.read().await;
            if !schemas.contains_key(&schema) {
                return Err(pliantdb_core::Error::SchemaNotRegistered(schema));
            }
        }

        let mut available_databases = self.data.available_databases.write().await;
        let admin = self.admin().await;
        if !admin
            .view::<database::ByName>()
            .with_key(name.to_ascii_lowercase())
            .query()
            .await?
            .is_empty()
        {
            return Err(pliantdb_core::Error::DatabaseNameAlreadyTaken(
                name.to_string(),
            ));
        }

        admin
            .collection::<DatabaseRecord>()
            .push(&connection::Database {
                name: name.to_string(),
                schema: schema.clone(),
            })
            .await?;
        available_databases.insert(name.to_string(), schema);

        Ok(())
    }

    async fn delete_database(&self, name: &str) -> Result<(), pliantdb_core::Error> {
        let mut available_databases = self.data.available_databases.write().await;

        let prefix = format!("{}::", name);
        let sled = self.data.sled.clone();
        tokio::task::spawn_blocking(move || {
            for name in sled.tree_names() {
                if name.starts_with(prefix.as_bytes()) {
                    sled.drop_tree(name)?;
                }
            }
            Result::<_, sled::Error>::Ok(())
        })
        .await
        .unwrap()
        .map_err(Error::from)?;

        let admin = self.admin().await;
        if let Some(entry) = admin
            .view::<database::ByName>()
            .with_key(name.to_ascii_lowercase())
            .query_with_docs()
            .await?
            .first()
        {
            admin.delete::<DatabaseRecord>(&entry.document).await?;
            available_databases.remove(name);

            Ok(())
        } else {
            return Err(pliantdb_core::Error::DatabaseNotFound(name.to_string()));
        }
    }

    async fn list_databases(&self) -> Result<Vec<connection::Database>, pliantdb_core::Error> {
        let available_databases = self.data.available_databases.read().await;
        Ok(available_databases
            .iter()
            .map(|(name, schema)| connection::Database {
                name: name.to_string(),
                schema: schema.clone(),
            })
            .collect())
    }

    async fn list_available_schemas(&self) -> Result<Vec<SchemaName>, pliantdb_core::Error> {
        let available_databases = self.data.available_databases.read().await;
        Ok(available_databases.values().unique().cloned().collect())
    }

    async fn create_user(&self, username: &str) -> Result<u64, pliantdb_core::Error> {
        let result = self
            .admin()
            .await
            .collection::<User>()
            .push(&User::default_with_username(username))
            .await?;
        Ok(result.id)
    }

    async fn set_user_password(
        &self,
        username: &str,
        password_request: pliantdb_core::custodian_password::RegistrationRequest,
    ) -> Result<pliantdb_core::custodian_password::RegistrationResponse, pliantdb_core::Error> {
        let admin = self.admin().await;
        let result = admin
            .view::<user::ByName>()
            .with_key(username.to_owned())
            .query_with_docs()
            .await
            .unwrap();
        if result.is_empty() {
            Err(pliantdb_core::Error::UserNotFound)
        } else {
            let config = PasswordConfig::load(&admin).await.unwrap();
            let (register, response) = ServerRegistration::register(&config, password_request)?;

            let mut doc =
                CollectionDocument::<User>::try_from(result.into_iter().next().unwrap().document)?;
            doc.contents.pending_password_change_state = Some(register);
            doc.update(&admin).await?;

            Ok(response)
        }
    }

    async fn finish_set_user_password(
        &self,
        username: &str,
        password_finalization: pliantdb_core::custodian_password::RegistrationFinalization,
    ) -> Result<(), pliantdb_core::Error> {
        let admin = self.admin().await;
        let result = admin
            .view::<user::ByName>()
            .with_key(username.to_owned())
            .query_with_docs()
            .await?;
        if result.is_empty() {
            Err(pliantdb_core::Error::UserNotFound)
        } else {
            let mut doc =
                CollectionDocument::<User>::try_from(result.into_iter().next().unwrap().document)?;
            if let Some(registration) = doc.contents.pending_password_change_state.take() {
                let file = registration.finish(password_finalization)?;
                doc.contents.password_hash = Some(file);
                doc.update(&admin).await?;

                Ok(())
            } else {
                Err(pliantdb_core::Error::Password(String::from(
                    "no existing state found",
                )))
            }
        }
    }
}

#[test]
fn name_validation_tests() {
    assert!(matches!(Storage::validate_name("azAZ09.-"), Ok(())));
    assert!(matches!(
        Storage::validate_name(".alphaunmericfirstrequired"),
        Err(Error::Core(pliantdb_core::Error::InvalidDatabaseName(_)))
    ));
    assert!(matches!(
        Storage::validate_name("-alphaunmericfirstrequired"),
        Err(Error::Core(pliantdb_core::Error::InvalidDatabaseName(_)))
    ));
    assert!(matches!(
        Storage::validate_name("\u{2661}"),
        Err(Error::Core(pliantdb_core::Error::InvalidDatabaseName(_)))
    ));
}

#[derive(Clone, Copy, Eq, PartialEq, Hash)]
pub struct StorageId(u64);

impl StorageId {
    pub const fn as_u64(self) -> u64 {
        self.0
    }
}

impl Debug for StorageId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // let formatted_length = format!();
        write!(f, "{:016x}", self.0)
    }
}

impl Display for StorageId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(self, f)
    }
}
