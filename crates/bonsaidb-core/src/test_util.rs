#![allow(clippy::missing_panics_doc)]

use std::{
    fmt::{Debug, Display},
    io::ErrorKind,
    ops::Deref,
    path::{Path, PathBuf},
    time::{Duration, Instant},
};

use itertools::Itertools;
use serde::{Deserialize, Serialize};

#[cfg(feature = "multiuser")]
use crate::admin::{PermissionGroup, Role, User};
use crate::{
    connection::{AccessPolicy, Connection, StorageConnection},
    document::{Document, KeyId},
    keyvalue::KeyValue,
    limits::{LIST_TRANSACTIONS_DEFAULT_RESULT_COUNT, LIST_TRANSACTIONS_MAX_RESULTS},
    schema::{
        view::{self, map::Mappings},
        Collection, CollectionDocument, CollectionName, InvalidNameError, MapResult, MappedValue,
        Name, NamedCollection, Schema, SchemaName, Schematic, View,
    },
    Error, ENCRYPTION_ENABLED,
};

#[derive(Serialize, Deserialize, Debug, PartialEq, Default, Clone)]
pub struct Basic {
    pub value: String,
    pub category: Option<String>,
    pub parent_id: Option<u64>,
    pub tags: Vec<String>,
}

impl Basic {
    pub fn new(value: impl Into<String>) -> Self {
        Self {
            value: value.into(),
            tags: Vec::default(),
            category: None,
            parent_id: None,
        }
    }

    pub fn with_category(mut self, category: impl Into<String>) -> Self {
        self.category = Some(category.into());
        self
    }

    pub fn with_tag(mut self, tag: impl Into<String>) -> Self {
        self.tags.push(tag.into());
        self
    }

    #[must_use]
    pub const fn with_parent_id(mut self, parent_id: u64) -> Self {
        self.parent_id = Some(parent_id);
        self
    }
}

impl Collection for Basic {
    fn collection_name() -> Result<CollectionName, InvalidNameError> {
        CollectionName::new("khonsulabs", "basic")
    }

    fn define_views(schema: &mut Schematic) -> Result<(), Error> {
        schema.define_view(BasicCount)?;
        schema.define_view(BasicByParentId)?;
        schema.define_view(BasicByTag)?;
        schema.define_view(BasicByCategory)
    }
}

#[cfg(feature = "json")]
#[derive(Serialize, Deserialize, Debug, PartialEq, Default, Clone)]
struct JsonBasic {
    pub value: u64,
}

#[cfg(feature = "json")]
impl Collection for JsonBasic {
    fn collection_name() -> Result<CollectionName, InvalidNameError> {
        CollectionName::new("khonsulabs", "json-basic")
    }

    fn define_views(_schema: &mut Schematic) -> Result<(), Error> {
        Ok(())
    }

    fn serializer() -> crate::schema::CollectionSerializer {
        crate::schema::CollectionSerializer::Json
    }
}

#[cfg(feature = "cbor")]
#[derive(Serialize, Deserialize, Debug, PartialEq, Default, Clone)]
struct CborBasic {
    pub value: u64,
}

#[cfg(feature = "cbor")]
impl Collection for CborBasic {
    fn collection_name() -> Result<CollectionName, InvalidNameError> {
        CollectionName::new("khonsulabs", "cbor-basic")
    }

    fn define_views(_schema: &mut Schematic) -> Result<(), Error> {
        Ok(())
    }

    fn serializer() -> crate::schema::CollectionSerializer {
        crate::schema::CollectionSerializer::Cbor
    }
}

#[cfg(feature = "bincode")]
#[derive(Serialize, Deserialize, Debug, PartialEq, Default, Clone)]
struct BincodeBasic {
    pub value: u64,
}

#[cfg(feature = "bincode")]
impl Collection for BincodeBasic {
    fn collection_name() -> Result<CollectionName, InvalidNameError> {
        CollectionName::new("khonsulabs", "bincode-basic")
    }

    fn define_views(_schema: &mut Schematic) -> Result<(), Error> {
        Ok(())
    }

    fn serializer() -> crate::schema::CollectionSerializer {
        crate::schema::CollectionSerializer::Bincode
    }
}

#[derive(Debug)]
pub struct BasicCount;

impl View for BasicCount {
    type Collection = Basic;
    type Key = ();
    type Value = usize;

    fn version(&self) -> u64 {
        0
    }

    fn name(&self) -> Result<Name, InvalidNameError> {
        Name::new("count")
    }

    fn map(&self, document: &Document<'_>) -> MapResult<Self::Key, Self::Value> {
        Ok(document.emit_key_and_value((), 1))
    }

    fn reduce(
        &self,
        mappings: &[MappedValue<Self::Key, Self::Value>],
        _rereduce: bool,
    ) -> Result<Self::Value, view::Error> {
        Ok(mappings.iter().map(|map| map.value).sum())
    }
}

#[derive(Debug)]
pub struct BasicByParentId;

impl View for BasicByParentId {
    type Collection = Basic;
    type Key = Option<u64>;
    type Value = usize;

    fn version(&self) -> u64 {
        1
    }

    fn name(&self) -> Result<Name, InvalidNameError> {
        Name::new("by-parent-id")
    }

    fn map(&self, document: &Document<'_>) -> MapResult<Self::Key, Self::Value> {
        let contents = document.contents::<Basic>()?;
        Ok(document.emit_key_and_value(contents.parent_id, 1))
    }

    fn reduce(
        &self,
        mappings: &[MappedValue<Self::Key, Self::Value>],
        _rereduce: bool,
    ) -> Result<Self::Value, view::Error> {
        Ok(mappings.iter().map(|map| map.value).sum())
    }
}

#[derive(Debug)]
pub struct BasicByCategory;

impl View for BasicByCategory {
    type Collection = Basic;
    type Key = String;
    type Value = usize;

    fn version(&self) -> u64 {
        0
    }

    fn name(&self) -> Result<Name, InvalidNameError> {
        Name::new("by-category")
    }

    fn map(&self, document: &Document<'_>) -> MapResult<Self::Key, Self::Value> {
        let contents = document.contents::<Basic>()?;
        if let Some(category) = &contents.category {
            Ok(document.emit_key_and_value(category.to_lowercase(), 1))
        } else {
            Ok(Mappings::none())
        }
    }

    fn reduce(
        &self,
        mappings: &[MappedValue<Self::Key, Self::Value>],
        _rereduce: bool,
    ) -> Result<Self::Value, view::Error> {
        Ok(mappings.iter().map(|map| map.value).sum())
    }
}

#[derive(Debug)]
pub struct BasicByTag;

impl View for BasicByTag {
    type Collection = Basic;
    type Key = String;
    type Value = usize;

    fn version(&self) -> u64 {
        0
    }

    fn name(&self) -> Result<Name, InvalidNameError> {
        Name::new("by-tag")
    }

    fn map(&self, document: &Document<'_>) -> MapResult<Self::Key, Self::Value> {
        let contents = document.contents::<Basic>()?;

        Ok(contents
            .tags
            .iter()
            .map(|tag| document.emit_key_and_value(tag.clone(), 1))
            .collect())
    }

    fn reduce(
        &self,
        mappings: &[MappedValue<Self::Key, Self::Value>],
        _rereduce: bool,
    ) -> Result<Self::Value, view::Error> {
        Ok(mappings.iter().map(|map| map.value).sum())
    }
}

#[derive(Debug)]
pub struct BasicByBrokenParentId;

impl View for BasicByBrokenParentId {
    type Collection = Basic;
    type Key = ();
    type Value = ();

    fn version(&self) -> u64 {
        0
    }

    fn name(&self) -> Result<Name, InvalidNameError> {
        Name::new("by-parent-id")
    }

    fn map(&self, document: &Document<'_>) -> MapResult<Self::Key, Self::Value> {
        Ok(document.emit())
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Default, Clone)]
pub struct EncryptedBasic {
    pub value: String,
    pub category: Option<String>,
    pub parent_id: Option<u64>,
}

impl EncryptedBasic {
    pub fn new(value: impl Into<String>) -> Self {
        Self {
            value: value.into(),
            category: None,
            parent_id: None,
        }
    }

    pub fn with_category(mut self, category: impl Into<String>) -> Self {
        self.category = Some(category.into());
        self
    }

    #[must_use]
    pub const fn with_parent_id(mut self, parent_id: u64) -> Self {
        self.parent_id = Some(parent_id);
        self
    }
}

impl Collection for EncryptedBasic {
    fn encryption_key() -> Option<KeyId> {
        if ENCRYPTION_ENABLED {
            Some(KeyId::Master)
        } else {
            None
        }
    }

    fn collection_name() -> Result<CollectionName, InvalidNameError> {
        CollectionName::new("khonsulabs", "encrypted-basic")
    }

    fn define_views(schema: &mut Schematic) -> Result<(), Error> {
        schema.define_view(EncryptedBasicCount)?;
        schema.define_view(EncryptedBasicByParentId)?;
        schema.define_view(EncryptedBasicByCategory)
    }
}

#[derive(Debug)]
pub struct EncryptedBasicCount;

impl View for EncryptedBasicCount {
    type Collection = EncryptedBasic;
    type Key = ();
    type Value = usize;

    fn version(&self) -> u64 {
        0
    }

    fn name(&self) -> Result<Name, InvalidNameError> {
        Name::new("count")
    }

    fn map(&self, document: &Document<'_>) -> MapResult<Self::Key, Self::Value> {
        Ok(document.emit_key_and_value((), 1))
    }

    fn reduce(
        &self,
        mappings: &[MappedValue<Self::Key, Self::Value>],
        _rereduce: bool,
    ) -> Result<Self::Value, view::Error> {
        Ok(mappings.iter().map(|map| map.value).sum())
    }
}

#[derive(Debug)]
pub struct EncryptedBasicByParentId;

impl View for EncryptedBasicByParentId {
    type Collection = EncryptedBasic;
    type Key = Option<u64>;
    type Value = usize;

    fn version(&self) -> u64 {
        1
    }

    fn name(&self) -> Result<Name, InvalidNameError> {
        Name::new("by-parent-id")
    }

    fn map(&self, document: &Document<'_>) -> MapResult<Self::Key, Self::Value> {
        let contents = document.contents::<EncryptedBasic>()?;
        Ok(document.emit_key_and_value(contents.parent_id, 1))
    }

    fn reduce(
        &self,
        mappings: &[MappedValue<Self::Key, Self::Value>],
        _rereduce: bool,
    ) -> Result<Self::Value, view::Error> {
        Ok(mappings.iter().map(|map| map.value).sum())
    }
}

#[derive(Debug)]
pub struct EncryptedBasicByCategory;

impl View for EncryptedBasicByCategory {
    type Collection = EncryptedBasic;
    type Key = String;
    type Value = usize;

    fn version(&self) -> u64 {
        0
    }

    fn name(&self) -> Result<Name, InvalidNameError> {
        Name::new("by-category")
    }

    fn map(&self, document: &Document<'_>) -> MapResult<Self::Key, Self::Value> {
        let contents = document.contents::<EncryptedBasic>()?;
        if let Some(category) = &contents.category {
            Ok(document.emit_key_and_value(category.to_lowercase(), 1))
        } else {
            Ok(Mappings::none())
        }
    }

    fn reduce(
        &self,
        mappings: &[MappedValue<Self::Key, Self::Value>],
        _rereduce: bool,
    ) -> Result<Self::Value, view::Error> {
        Ok(mappings.iter().map(|map| map.value).sum())
    }
}

#[derive(Debug)]
pub struct BasicSchema;

impl Schema for BasicSchema {
    fn schema_name() -> Result<SchemaName, InvalidNameError> {
        SchemaName::new("khonsulabs", "basic")
    }

    fn define_collections(schema: &mut Schematic) -> Result<(), Error> {
        schema.define_collection::<Basic>()?;
        #[cfg(feature = "json")]
        schema.define_collection::<JsonBasic>()?;
        #[cfg(feature = "cbor")]
        schema.define_collection::<CborBasic>()?;
        #[cfg(feature = "bincode")]
        schema.define_collection::<BincodeBasic>()?;
        schema.define_collection::<EncryptedBasic>()?;
        schema.define_collection::<Unique>()
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Default)]
pub struct Unique {
    pub value: String,
}

impl Unique {
    pub fn new(value: impl Display) -> Self {
        Self {
            value: value.to_string(),
        }
    }
}

impl Collection for Unique {
    fn collection_name() -> Result<CollectionName, InvalidNameError> {
        CollectionName::new("khonsulabs", "unique")
    }

    fn define_views(schema: &mut Schematic) -> Result<(), Error> {
        schema.define_view(UniqueValue)
    }
}

#[derive(Debug)]
pub struct UniqueValue;

impl View for UniqueValue {
    type Collection = Unique;
    type Key = String;
    type Value = ();

    fn unique(&self) -> bool {
        true
    }

    fn version(&self) -> u64 {
        1
    }

    fn name(&self) -> Result<Name, InvalidNameError> {
        Name::new("unique-value")
    }

    fn map(&self, document: &Document<'_>) -> MapResult<Self::Key, Self::Value> {
        let entry = document.contents::<Unique>()?;
        Ok(document.emit_key(entry.value))
    }
}

impl NamedCollection for Unique {
    type ByNameView = UniqueValue;
}

pub struct TestDirectory(pub PathBuf);

impl TestDirectory {
    pub fn new<S: AsRef<Path>>(name: S) -> Self {
        let path = std::env::temp_dir().join(name);
        if path.exists() {
            std::fs::remove_dir_all(&path).expect("error clearing temporary directory");
        }
        Self(path)
    }
}

impl Drop for TestDirectory {
    fn drop(&mut self) {
        if let Err(err) = std::fs::remove_dir_all(&self.0) {
            if err.kind() != ErrorKind::NotFound {
                eprintln!("Failed to clean up temporary folder: {:?}", err);
            }
        }
    }
}

impl AsRef<Path> for TestDirectory {
    fn as_ref(&self) -> &Path {
        &self.0
    }
}

impl Deref for TestDirectory {
    type Target = PathBuf;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug)]
pub struct BasicCollectionWithNoViews;

impl Collection for BasicCollectionWithNoViews {
    fn collection_name() -> Result<CollectionName, InvalidNameError> {
        Basic::collection_name()
    }

    fn define_views(_schema: &mut Schematic) -> Result<(), Error> {
        Ok(())
    }
}

#[derive(Debug)]
pub struct BasicCollectionWithOnlyBrokenParentId;

impl Collection for BasicCollectionWithOnlyBrokenParentId {
    fn collection_name() -> Result<CollectionName, InvalidNameError> {
        Basic::collection_name()
    }

    fn define_views(schema: &mut Schematic) -> Result<(), Error> {
        schema.define_view(BasicByBrokenParentId)
    }
}

#[derive(Debug)]
pub struct UnassociatedCollection;

impl Collection for UnassociatedCollection {
    fn collection_name() -> Result<CollectionName, InvalidNameError> {
        CollectionName::new("khonsulabs", "unassociated")
    }

    fn define_views(_schema: &mut Schematic) -> Result<(), Error> {
        Ok(())
    }
}

#[derive(Copy, Clone, Debug)]
pub enum HarnessTest {
    ServerConnectionTests = 1,
    StoreRetrieveUpdate,
    CollectionSerialization,
    NotFound,
    Conflict,
    BadUpdate,
    NoUpdate,
    GetMultiple,
    List,
    ListTransactions,
    ViewQuery,
    UnassociatedCollection,
    Compact,
    ViewUpdate,
    ViewMultiEmit,
    ViewAccessPolicies,
    Encryption,
    UniqueViews,
    NamedCollection,
    PubSubSimple,
    UserManagement,
    PubSubMultipleSubscribers,
    PubSubDropAndSend,
    PubSubUnsubscribe,
    PubSubDropCleanup,
    PubSubPublishAll,
    KvBasic,
    KvSet,
    KvIncrementDecrement,
    KvExpiration,
    KvDeleteExpire,
    KvTransactions,
}

impl HarnessTest {
    #[must_use]
    pub const fn port(self, base: u16) -> u16 {
        base + self as u16
    }
}

impl Display for HarnessTest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(&self, f)
    }
}

/// Compares two f64's accounting for the epsilon.
#[macro_export]
macro_rules! assert_f64_eq {
    ($a:expr, $b:expr) => {{
        let a: f64 = $a;
        let b: f64 = $b;
        if (a - b).abs() > f64::EPSILON {
            panic!("{:?} <> {:?}", a, b);
        }
    }};
}

/// Creates a test suite that tests methods available on [`Connection`]
#[macro_export]
macro_rules! define_connection_test_suite {
    ($harness:ident) => {
        #[tokio::test]
        async fn server_connection_tests() -> anyhow::Result<()> {
            let harness =
                $harness::new($crate::test_util::HarnessTest::ServerConnectionTests).await?;
            let db = harness.server();
            $crate::test_util::basic_server_connection_tests(
                db.clone(),
                &format!("server-connection-tests-{}", $harness::server_name()),
            )
            .await?;
            harness.shutdown().await
        }

        #[tokio::test]
        async fn store_retrieve_update_delete() -> anyhow::Result<()> {
            let harness =
                $harness::new($crate::test_util::HarnessTest::StoreRetrieveUpdate).await?;
            let db = harness.connect().await?;
            $crate::test_util::store_retrieve_update_delete_tests(&db).await?;
            harness.shutdown().await
        }

        #[tokio::test]
        async fn collection_serialization() -> anyhow::Result<()> {
            let harness =
                $harness::new($crate::test_util::HarnessTest::CollectionSerialization).await?;
            let db = harness.connect().await?;
            $crate::test_util::collection_serialization_tests(&db).await?;
            harness.shutdown().await
        }

        #[tokio::test]
        async fn not_found() -> anyhow::Result<()> {
            let harness = $harness::new($crate::test_util::HarnessTest::NotFound).await?;
            let db = harness.connect().await?;

            $crate::test_util::not_found_tests(&db).await?;
            harness.shutdown().await
        }

        #[tokio::test]
        async fn conflict() -> anyhow::Result<()> {
            let harness = $harness::new($crate::test_util::HarnessTest::Conflict).await?;
            let db = harness.connect().await?;

            $crate::test_util::conflict_tests(&db).await?;
            harness.shutdown().await
        }

        #[tokio::test]
        async fn bad_update() -> anyhow::Result<()> {
            let harness = $harness::new($crate::test_util::HarnessTest::BadUpdate).await?;
            let db = harness.connect().await?;

            $crate::test_util::bad_update_tests(&db).await?;
            harness.shutdown().await
        }

        #[tokio::test]
        async fn no_update() -> anyhow::Result<()> {
            let harness = $harness::new($crate::test_util::HarnessTest::NoUpdate).await?;
            let db = harness.connect().await?;

            $crate::test_util::no_update_tests(&db).await?;
            harness.shutdown().await
        }

        #[tokio::test]
        async fn get_multiple() -> anyhow::Result<()> {
            let harness = $harness::new($crate::test_util::HarnessTest::GetMultiple).await?;
            let db = harness.connect().await?;

            $crate::test_util::get_multiple_tests(&db).await?;
            harness.shutdown().await
        }

        #[tokio::test]
        async fn list() -> anyhow::Result<()> {
            let harness = $harness::new($crate::test_util::HarnessTest::List).await?;
            let db = harness.connect().await?;

            $crate::test_util::list_tests(&db).await?;
            harness.shutdown().await
        }

        #[tokio::test]
        async fn list_transactions() -> anyhow::Result<()> {
            let harness = $harness::new($crate::test_util::HarnessTest::ListTransactions).await?;
            let db = harness.connect().await?;

            $crate::test_util::list_transactions_tests(&db).await?;
            harness.shutdown().await
        }

        #[tokio::test]
        async fn view_query() -> anyhow::Result<()> {
            let harness = $harness::new($crate::test_util::HarnessTest::ViewQuery).await?;
            let db = harness.connect().await?;

            $crate::test_util::view_query_tests(&db).await?;
            harness.shutdown().await
        }

        #[tokio::test]
        async fn unassociated_collection() -> anyhow::Result<()> {
            let harness =
                $harness::new($crate::test_util::HarnessTest::UnassociatedCollection).await?;
            let db = harness.connect().await?;

            $crate::test_util::unassociated_collection_tests(&db).await?;
            harness.shutdown().await
        }

        #[tokio::test]
        async fn view_update() -> anyhow::Result<()> {
            let harness = $harness::new($crate::test_util::HarnessTest::ViewUpdate).await?;
            let db = harness.connect().await?;

            $crate::test_util::view_update_tests(&db).await?;
            harness.shutdown().await
        }

        #[tokio::test]
        async fn view_multi_emit() -> anyhow::Result<()> {
            let harness = $harness::new($crate::test_util::HarnessTest::ViewMultiEmit).await?;
            let db = harness.connect().await?;

            $crate::test_util::view_multi_emit_tests(&db).await?;
            harness.shutdown().await
        }

        #[tokio::test]
        async fn view_access_policies() -> anyhow::Result<()> {
            let harness = $harness::new($crate::test_util::HarnessTest::ViewAccessPolicies).await?;
            let db = harness.connect().await?;

            $crate::test_util::view_access_policy_tests(&db).await?;
            harness.shutdown().await
        }

        #[tokio::test]
        async fn unique_views() -> anyhow::Result<()> {
            let harness = $harness::new($crate::test_util::HarnessTest::UniqueViews).await?;
            let db = harness.connect().await?;

            $crate::test_util::unique_view_tests(&db).await?;
            harness.shutdown().await
        }

        #[tokio::test]
        async fn named_collection() -> anyhow::Result<()> {
            let harness = $harness::new($crate::test_util::HarnessTest::NamedCollection).await?;
            let db = harness.connect().await?;

            $crate::test_util::named_collection_tests(&db).await?;
            harness.shutdown().await
        }

        #[tokio::test]
        #[cfg(any(feature = "multiuser", feature = "local-multiuser", feature = "server"))]
        async fn user_management() -> anyhow::Result<()> {
            let harness = $harness::new($crate::test_util::HarnessTest::UserManagement).await?;
            let _db = harness.connect().await?;
            let server = harness.server();
            let admin = server
                .database::<$crate::admin::Admin>($crate::admin::ADMIN_DATABASE_NAME)
                .await?;

            $crate::test_util::user_management_tests(
                &admin,
                server.clone(),
                $harness::server_name(),
            )
            .await?;
            harness.shutdown().await
        }

        #[tokio::test]
        async fn compaction() -> anyhow::Result<()> {
            let harness = $harness::new($crate::test_util::HarnessTest::Compact).await?;
            let db = harness.connect().await?;

            $crate::test_util::compaction_tests(&db).await?;
            harness.shutdown().await
        }
    };
}

pub async fn store_retrieve_update_delete_tests<C: Connection>(db: &C) -> anyhow::Result<()> {
    let original_value = Basic::new("initial_value");
    let collection = db.collection::<Basic>();
    let header = collection.push(&original_value).await?;

    let mut doc = collection
        .get(header.id)
        .await?
        .expect("couldn't retrieve stored item");
    let mut value = doc.contents::<Basic>()?;
    assert_eq!(original_value, value);
    let old_revision = doc.header.revision.clone();

    // Update the value
    value.value = String::from("updated_value");
    doc.set_contents(&value)?;
    db.update::<Basic>(&mut doc).await?;

    // update should cause the revision to be changed
    assert_ne!(doc.header.revision, old_revision);

    // Check the value in the database to ensure it has the new document
    let doc = collection
        .get(header.id)
        .await?
        .expect("couldn't retrieve stored item");
    assert_eq!(doc.contents::<Basic>()?, value);

    // These operations should have created two transactions with one change each
    let transactions = db.list_executed_transactions(None, None).await?;
    assert_eq!(transactions.len(), 2);
    assert!(transactions[0].id < transactions[1].id);
    for transaction in &transactions {
        let changed_documents = transaction
            .changes
            .documents()
            .expect("incorrect transaction type");
        assert_eq!(changed_documents.len(), 1);
        assert_eq!(changed_documents[0].collection, Basic::collection_name()?);
        assert_eq!(changed_documents[0].id, header.id);
        assert!(!changed_documents[0].deleted);
    }

    db.delete::<Basic>(&doc).await?;
    assert!(collection.get(header.id).await?.is_none());
    let transactions = db
        .list_executed_transactions(Some(transactions.last().as_ref().unwrap().id + 1), None)
        .await?;
    assert_eq!(transactions.len(), 1);
    let transaction = transactions.first().unwrap();
    let changed_documents = transaction
        .changes
        .documents()
        .expect("incorrect transaction type");
    assert_eq!(changed_documents.len(), 1);
    assert_eq!(changed_documents[0].collection, Basic::collection_name()?);
    assert_eq!(changed_documents[0].id, header.id);
    assert!(changed_documents[0].deleted);

    // Use the Collection interface
    let mut doc = original_value.clone().insert_into(db).await?;
    doc.contents.category = Some(String::from("updated"));
    doc.update(db).await?;
    let reloaded = Basic::get(doc.header.id, db).await?.unwrap();
    assert_eq!(doc.contents, reloaded.contents);

    // Test Connection::insert with a specified id
    let doc = Document::with_contents(42, &Basic::new("42"))?;
    let document_42 = db
        .insert::<Basic>(Some(doc.id), doc.contents.to_vec())
        .await?;
    assert_eq!(document_42.id, 42);

    // Test that inserting a document with the same ID restuls in a conflict:
    let conflict_err = db
        .insert::<Basic>(Some(doc.id), doc.contents.to_vec())
        .await;
    assert!(matches!(conflict_err, Err(Error::DocumentConflict(..))));

    Ok(())
}

#[cfg_attr(
    not(any(feature = "json", feature = "cbor", feature = "bincode")),
    allow(clippy::unused_async, unused_variables)
)]
pub async fn collection_serialization_tests<C: Connection>(db: &C) -> anyhow::Result<()> {
    #[cfg(feature = "json")]
    {
        let header = db
            .collection::<JsonBasic>()
            .push(&JsonBasic { value: 1 })
            .await?;
        let doc = db
            .get::<JsonBasic>(header.id)
            .await?
            .expect("failed to get json doc");
        assert!(serde_json::from_slice::<JsonBasic>(&doc.contents).is_ok());
        let deserialized = doc.contents::<JsonBasic>().unwrap();
        assert_eq!(deserialized.value, 1);

        // Use the CollectionDocument interface
        let inserted = JsonBasic { value: 2 }.insert_into(db).await?;
        let retrieved = JsonBasic::get(inserted.id, db).await?;
        assert_eq!(Some(inserted), retrieved);
    }
    #[cfg(feature = "cbor")]
    {
        let header = db
            .collection::<CborBasic>()
            .push(&CborBasic { value: 1 })
            .await?;
        let doc = db
            .get::<CborBasic>(header.id)
            .await?
            .expect("failed to get cbor doc");
        assert!(ciborium::de::from_reader::<CborBasic, _>(&doc.contents[..]).is_ok());
        let deserialized = doc.contents::<CborBasic>().unwrap();
        assert_eq!(deserialized.value, 1);

        // Use the CollectionDocument interface
        let inserted = CborBasic { value: 2 }.insert_into(db).await?;
        let retrieved = CborBasic::get(inserted.id, db).await?;
        assert_eq!(Some(inserted), retrieved);
    }
    #[cfg(feature = "bincode")]
    {
        let header = db
            .collection::<BincodeBasic>()
            .push(&BincodeBasic { value: 1 })
            .await?;
        let doc = db
            .get::<BincodeBasic>(header.id)
            .await?
            .expect("failed to get json doc");
        assert!(bincode::deserialize::<BincodeBasic>(&doc.contents).is_ok());
        let deserialized = doc.contents::<BincodeBasic>().unwrap();
        assert_eq!(deserialized.value, 1);

        // Use the CollectionDocument interface
        let inserted = BincodeBasic { value: 2 }.insert_into(db).await?;
        let retrieved = BincodeBasic::get(inserted.id, db).await?;
        assert_eq!(Some(inserted), retrieved);
    }

    Ok(())
}

pub async fn not_found_tests<C: Connection>(db: &C) -> anyhow::Result<()> {
    assert!(db.collection::<Basic>().get(1).await?.is_none());

    assert!(db.last_transaction_id().await?.is_none());

    Ok(())
}

pub async fn conflict_tests<C: Connection>(db: &C) -> anyhow::Result<()> {
    let original_value = Basic::new("initial_value");
    let collection = db.collection::<Basic>();
    let header = collection.push(&original_value).await?;

    let mut doc = collection
        .get(header.id)
        .await?
        .expect("couldn't retrieve stored item");
    let mut value = doc.contents::<Basic>()?;
    value.value = String::from("updated_value");
    doc.set_contents(&value)?;
    db.update::<Basic>(&mut doc).await?;

    // To generate a conflict, let's try to do the same update again by
    // reverting the header
    doc.header = header;
    match db
        .update::<Basic>(&mut doc)
        .await
        .expect_err("conflict should have generated an error")
    {
        Error::DocumentConflict(collection, id) => {
            assert_eq!(collection, Basic::collection_name()?);
            assert_eq!(id, doc.header.id);
        }
        other => return Err(anyhow::Error::from(other)),
    }

    // Now, let's use the CollectionDocument API to modify the document through a refetch.
    let mut doc = CollectionDocument::<Basic>::try_from(doc)?;
    doc.modify(db, |doc| {
        doc.contents.value = String::from("modify worked");
    })
    .await?;
    assert_eq!(doc.contents.value, "modify worked");
    let doc = Basic::get(doc.id, db).await?.unwrap();
    assert_eq!(doc.contents.value, "modify worked");

    Ok(())
}

pub async fn bad_update_tests<C: Connection>(db: &C) -> anyhow::Result<()> {
    let mut doc = Document::with_contents(1, &Basic::default())?;
    match db.update::<Basic>(&mut doc).await {
        Err(Error::DocumentNotFound(collection, id)) => {
            assert_eq!(collection, Basic::collection_name()?);
            assert_eq!(id, 1);
            Ok(())
        }
        other => panic!("expected DocumentNotFound from update but got: {:?}", other),
    }
}

pub async fn no_update_tests<C: Connection>(db: &C) -> anyhow::Result<()> {
    let original_value = Basic::new("initial_value");
    let collection = db.collection::<Basic>();
    let header = collection.push(&original_value).await?;

    let mut doc = collection
        .get(header.id)
        .await?
        .expect("couldn't retrieve stored item");
    db.update::<Basic>(&mut doc).await?;

    assert_eq!(doc.header, header);

    Ok(())
}

pub async fn get_multiple_tests<C: Connection>(db: &C) -> anyhow::Result<()> {
    let collection = db.collection::<Basic>();
    let doc1_value = Basic::new("initial_value");
    let doc1 = collection.push(&doc1_value).await?;

    let doc2_value = Basic::new("second_value");
    let doc2 = collection.push(&doc2_value).await?;

    let both_docs = db.get_multiple::<Basic>(&[doc1.id, doc2.id]).await?;
    assert_eq!(both_docs.len(), 2);

    let out_of_order = db.get_multiple::<Basic>(&[doc2.id, doc1.id]).await?;
    assert_eq!(out_of_order.len(), 2);

    // The order of get_multiple isn't guaranteed, so these two checks are done
    // with iterators instead of direct indexing
    let doc1 = both_docs
        .iter()
        .find(|doc| doc.header.id == doc1.id)
        .expect("Couldn't find doc1");
    let doc1 = doc1.contents::<Basic>()?;
    assert_eq!(doc1.value, doc1_value.value);
    let doc2 = both_docs
        .iter()
        .find(|doc| doc.header.id == doc2.id)
        .expect("Couldn't find doc2");
    let doc2 = doc2.contents::<Basic>()?;
    assert_eq!(doc2.value, doc2_value.value);

    Ok(())
}

pub async fn list_tests<C: Connection>(db: &C) -> anyhow::Result<()> {
    let collection = db.collection::<Basic>();
    let doc1_value = Basic::new("initial_value");
    let doc1 = collection.push(&doc1_value).await?;

    let doc2_value = Basic::new("second_value");
    let doc2 = collection.push(&doc2_value).await?;

    let both_docs = collection.list(doc1.id..=doc2.id).await?;
    assert_eq!(both_docs.len(), 2);

    let doc1_contents = both_docs[0].contents::<Basic>()?;
    assert_eq!(doc1_contents.value, doc1_value.value);
    let doc2_contents = both_docs[1].contents::<Basic>()?;
    assert_eq!(doc2_contents.value, doc2_value.value);

    let one_doc = collection.list(doc1.id..doc2.id).await?;
    assert_eq!(one_doc.len(), 1);

    let limited = collection
        .list(doc1.id..=doc2.id)
        .limit(1)
        .descending()
        .await?;
    assert_eq!(limited.len(), 1);
    let limited = limited[0].contents::<Basic>()?;
    assert_eq!(limited.value, doc2_contents.value);

    Ok(())
}

pub async fn list_transactions_tests<C: Connection>(db: &C) -> anyhow::Result<()> {
    let collection = db.collection::<Basic>();

    // create LIST_TRANSACTIONS_MAX_RESULTS + 1 items, giving us just enough
    // transactions to test the edge cases of `list_transactions`
    futures::future::join_all(
        (0..=(LIST_TRANSACTIONS_MAX_RESULTS))
            .map(|_| async { collection.push(&Basic::default()).await.unwrap() }),
    )
    .await;

    // Test defaults
    let transactions = db.list_executed_transactions(None, None).await?;
    assert_eq!(transactions.len(), LIST_TRANSACTIONS_DEFAULT_RESULT_COUNT);

    // Test max results limit
    let transactions = db
        .list_executed_transactions(None, Some(LIST_TRANSACTIONS_MAX_RESULTS + 1))
        .await?;
    assert_eq!(transactions.len(), LIST_TRANSACTIONS_MAX_RESULTS);

    // Test requesting 0 items
    let transactions = db.list_executed_transactions(None, Some(0)).await?;
    assert!(transactions.is_empty());

    // Test doing a loop fetching until we get no more results
    let mut transactions = Vec::new();
    let mut starting_id = None;
    loop {
        let chunk = db
            .list_executed_transactions(starting_id, Some(100))
            .await?;
        if chunk.is_empty() {
            break;
        }

        let max_id = chunk.last().map(|tx| tx.id).unwrap();
        starting_id = Some(max_id + 1);
        transactions.extend(chunk);
    }

    assert_eq!(transactions.len(), LIST_TRANSACTIONS_MAX_RESULTS + 1);

    Ok(())
}

pub async fn view_query_tests<C: Connection>(db: &C) -> anyhow::Result<()> {
    let collection = db.collection::<Basic>();
    let a = collection.push(&Basic::new("A")).await?;
    let b = collection.push(&Basic::new("B")).await?;
    let a_child = collection
        .push(
            &Basic::new("A.1")
                .with_parent_id(a.id)
                .with_category("Alpha"),
        )
        .await?;
    collection
        .push(&Basic::new("B.1").with_parent_id(b.id).with_category("Beta"))
        .await?;
    collection
        .push(&Basic::new("B.2").with_parent_id(b.id).with_category("beta"))
        .await?;

    let a_children = db
        .view::<BasicByParentId>()
        .with_key(Some(a.id))
        .query()
        .await?;
    assert_eq!(a_children.len(), 1);

    let a_children = db
        .view::<BasicByParentId>()
        .with_key(Some(a.id))
        .query_with_docs()
        .await?;
    assert_eq!(a_children.len(), 1);
    assert_eq!(a_children[0].document.header.id, a_child.id);

    let b_children = db
        .view::<BasicByParentId>()
        .with_key(Some(b.id))
        .query()
        .await?;
    assert_eq!(b_children.len(), 2);

    let a_and_b_children = db
        .view::<BasicByParentId>()
        .with_keys([Some(a.id), Some(b.id)])
        .query()
        .await?;
    assert_eq!(a_and_b_children.len(), 3);

    // Test out of order keys
    let a_and_b_children = db
        .view::<BasicByParentId>()
        .with_keys([Some(b.id), Some(a.id)])
        .query()
        .await?;
    assert_eq!(a_and_b_children.len(), 3);

    let has_parent = db
        .view::<BasicByParentId>()
        .with_key_range(Some(0)..=Some(u64::MAX))
        .query()
        .await?;
    assert_eq!(has_parent.len(), 3);
    // Verify the result is sorted ascending
    assert!(has_parent
        .windows(2)
        .all(|window| window[0].key <= window[1].key));

    // Test limiting and descending order
    let last_with_parent = db
        .view::<BasicByParentId>()
        .with_key_range(Some(0)..=Some(u64::MAX))
        .descending()
        .limit(1)
        .query()
        .await?;
    assert_eq!(last_with_parent.iter().map(|m| m.key).unique().count(), 1);
    assert_eq!(last_with_parent[0].key, has_parent[2].key);

    let items_with_categories = db.view::<BasicByCategory>().query().await?;
    assert_eq!(items_with_categories.len(), 3);

    // Test deleting
    let deleted_count = db
        .view::<BasicByParentId>()
        .with_key(Some(b.id))
        .delete_docs()
        .await?;
    assert_eq!(b_children.len() as u64, deleted_count);
    assert_eq!(
        db.view::<BasicByParentId>()
            .with_key(Some(b.id))
            .query()
            .await?
            .len(),
        0
    );

    Ok(())
}

pub async fn unassociated_collection_tests<C: Connection>(db: &C) -> anyhow::Result<()> {
    let result = db
        .collection::<UnassociatedCollection>()
        .push(&Basic::default())
        .await;
    match result {
        Err(Error::CollectionNotFound) => {}
        other => unreachable!("unexpected result: {:?}", other),
    }

    Ok(())
}

pub async fn view_update_tests<C: Connection>(db: &C) -> anyhow::Result<()> {
    let collection = db.collection::<Basic>();
    let a = collection.push(&Basic::new("A")).await?;

    let a_children = db
        .view::<BasicByParentId>()
        .with_key(Some(a.id))
        .query()
        .await?;
    assert_eq!(a_children.len(), 0);
    // The reduce function of `BasicByParentId` acts as a "count" of records.
    assert_eq!(
        db.view::<BasicByParentId>()
            .with_key(Some(a.id))
            .reduce()
            .await?,
        0
    );

    // Test inserting a new record and the view being made available
    let a_child = collection
        .push(
            &Basic::new("A.1")
                .with_parent_id(a.id)
                .with_category("Alpha"),
        )
        .await?;

    let a_children = db
        .view::<BasicByParentId>()
        .with_key(Some(a.id))
        .query()
        .await?;
    assert_eq!(a_children.len(), 1);
    assert_eq!(
        db.view::<BasicByParentId>()
            .with_key(Some(a.id))
            .reduce()
            .await?,
        1
    );

    // Verify reduce_grouped matches our expectations.
    assert_eq!(
        db.view::<BasicByParentId>().reduce_grouped().await?,
        vec![
            MappedValue {
                key: None,
                value: 1,
            },
            MappedValue {
                key: Some(a.id),
                value: 1,
            },
        ]
    );

    // Test updating the record and the view being updated appropriately
    let mut doc = db.collection::<Basic>().get(a_child.id).await?.unwrap();
    let mut basic = doc.contents::<Basic>()?;
    basic.parent_id = None;
    doc.set_contents(&basic)?;
    db.update::<Basic>(&mut doc).await?;

    let a_children = db
        .view::<BasicByParentId>()
        .with_key(Some(a.id))
        .query()
        .await?;
    assert_eq!(a_children.len(), 0);
    assert_eq!(
        db.view::<BasicByParentId>()
            .with_key(Some(a.id))
            .reduce()
            .await?,
        0
    );
    assert_eq!(db.view::<BasicByParentId>().reduce().await?, 2);

    // Test deleting a record and ensuring it goes away
    db.delete::<Basic>(&doc).await?;

    let all_entries = db.view::<BasicByParentId>().query().await?;
    assert_eq!(all_entries.len(), 1);

    // Verify reduce_grouped matches our expectations.
    assert_eq!(
        db.view::<BasicByParentId>().reduce_grouped().await?,
        vec![MappedValue {
            key: None,
            value: 1,
        },]
    );

    Ok(())
}

pub async fn view_multi_emit_tests<C: Connection>(db: &C) -> anyhow::Result<()> {
    let mut a = Basic::new("A")
        .with_tag("red")
        .with_tag("green")
        .insert_into(db)
        .await?;
    let mut b = Basic::new("B")
        .with_tag("blue")
        .with_tag("green")
        .insert_into(db)
        .await?;

    assert_eq!(
        db.view::<BasicByTag>()
            .with_key(String::from("green"))
            .query()
            .await?
            .len(),
        2
    );

    assert_eq!(
        db.view::<BasicByTag>()
            .with_key(String::from("red"))
            .query()
            .await?
            .len(),
        1
    );

    assert_eq!(
        db.view::<BasicByTag>()
            .with_key(String::from("blue"))
            .query()
            .await?
            .len(),
        1
    );

    // Change tags
    a.contents.tags = vec![String::from("red"), String::from("blue")];
    a.update(db).await?;

    assert_eq!(
        db.view::<BasicByTag>()
            .with_key(String::from("green"))
            .query()
            .await?
            .len(),
        1
    );

    assert_eq!(
        db.view::<BasicByTag>()
            .with_key(String::from("red"))
            .query()
            .await?
            .len(),
        1
    );

    assert_eq!(
        db.view::<BasicByTag>()
            .with_key(String::from("blue"))
            .query()
            .await?
            .len(),
        2
    );
    b.contents.tags.clear();
    b.update(db).await?;

    assert_eq!(
        db.view::<BasicByTag>()
            .with_key(String::from("green"))
            .query()
            .await?
            .len(),
        0
    );

    assert_eq!(
        db.view::<BasicByTag>()
            .with_key(String::from("red"))
            .query()
            .await?
            .len(),
        1
    );

    assert_eq!(
        db.view::<BasicByTag>()
            .with_key(String::from("blue"))
            .query()
            .await?
            .len(),
        1
    );

    Ok(())
}

pub async fn view_access_policy_tests<C: Connection>(db: &C) -> anyhow::Result<()> {
    let collection = db.collection::<Basic>();
    let a = collection.push(&Basic::new("A")).await?;

    // Test inserting a record that should match the view, but ask for it to be
    // NoUpdate. Verify we get no matches.
    collection
        .push(
            &Basic::new("A.1")
                .with_parent_id(a.id)
                .with_category("Alpha"),
        )
        .await?;

    let a_children = db
        .view::<BasicByParentId>()
        .with_key(Some(a.id))
        .with_access_policy(AccessPolicy::NoUpdate)
        .query()
        .await?;
    assert_eq!(a_children.len(), 0);

    tokio::time::sleep(Duration::from_millis(20)).await;

    // Verify the view still have no value, but this time ask for it to be
    // updated after returning
    let a_children = db
        .view::<BasicByParentId>()
        .with_key(Some(a.id))
        .with_access_policy(AccessPolicy::UpdateAfter)
        .query()
        .await?;
    assert_eq!(a_children.len(), 0);

    // Waiting on background jobs can be unreliable in a CI environment
    for _ in 0..10_u8 {
        tokio::time::sleep(Duration::from_millis(20)).await;

        // Now, the view should contain the entry.
        let a_children = db
            .view::<BasicByParentId>()
            .with_key(Some(a.id))
            .with_access_policy(AccessPolicy::NoUpdate)
            .query()
            .await?;
        if a_children.len() == 1 {
            return Ok(());
        }
    }
    panic!("view never updated")
}

pub async fn unique_view_tests<C: Connection>(db: &C) -> anyhow::Result<()> {
    let first_doc = db.collection::<Unique>().push(&Unique::new("1")).await?;

    if let Err(Error::UniqueKeyViolation {
        view,
        existing_document,
        conflicting_document,
    }) = db.collection::<Unique>().push(&Unique::new("1")).await
    {
        assert_eq!(view, UniqueValue.view_name()?);
        assert_eq!(existing_document.id, first_doc.id);
        // We can't predict the conflicting document id since it's generated
        // inside of the transaction, but we can assert that it's different than
        // the document that was previously stored.
        assert_ne!(conflicting_document, existing_document);
    } else {
        unreachable!("unique key violation not triggered");
    }

    let second_doc = db.collection::<Unique>().push(&Unique::new("2")).await?;
    let mut second_doc = db.collection::<Unique>().get(second_doc.id).await?.unwrap();
    let mut contents = second_doc.contents::<Unique>()?;
    contents.value = String::from("1");
    second_doc.set_contents(&contents)?;
    if let Err(Error::UniqueKeyViolation {
        view,
        existing_document,
        conflicting_document,
    }) = db.update::<Unique>(&mut second_doc).await
    {
        assert_eq!(view, UniqueValue.view_name()?);
        assert_eq!(existing_document.id, first_doc.id);
        assert_eq!(conflicting_document.id, second_doc.header.id);
    } else {
        unreachable!("unique key violation not triggered");
    }

    Ok(())
}

pub async fn named_collection_tests<C: Connection>(db: &C) -> anyhow::Result<()> {
    Unique::new("0").insert_into(db).await?;
    let original_entry = Unique::entry("1", db)
        .update_with(|_existing: &mut Unique| unreachable!())
        .or_insert_with(|| Unique::new("1"))
        .await?
        .expect("Document not inserted");

    let updated = Unique::entry("1", db)
        .update_with(|existing: &mut Unique| {
            existing.value = String::from("2");
        })
        .or_insert_with(|| unreachable!())
        .await?
        .unwrap();
    assert_eq!(original_entry.id, updated.id);
    assert_ne!(original_entry.contents.value, updated.contents.value);

    let retrieved = Unique::entry("2", db).await?.unwrap();
    assert_eq!(retrieved.contents.value, updated.contents.value);

    let conflict = Unique::entry("2", db)
        .update_with(|existing: &mut Unique| {
            existing.value = String::from("0");
        })
        .await;
    assert!(matches!(conflict, Err(Error::UniqueKeyViolation { .. })));

    Ok(())
}

pub async fn compaction_tests<C: Connection + KeyValue>(db: &C) -> anyhow::Result<()> {
    let original_value = Basic::new("initial_value");
    let collection = db.collection::<Basic>();
    collection.push(&original_value).await?;

    // Test a collection compaction
    db.compact_collection::<Basic>().await?;

    // Test the key value store compaction
    db.set_key("foo", &1_u32).await?;
    db.compact_key_value_store().await?;

    // Compact everything... again...
    db.compact().await?;

    Ok(())
}

#[cfg(feature = "multiuser")]
pub async fn user_management_tests<C: Connection, S: StorageConnection>(
    admin: &C,
    server: S,
    server_name: &str,
) -> anyhow::Result<()> {
    let username = format!("user-management-tests-{}", server_name);
    let user_id = server.create_user(&username).await?;
    // Test the default created user state.
    {
        let user = User::get(user_id, admin)
            .await
            .unwrap()
            .expect("user not found");
        assert_eq!(user.contents.username, username);
        assert!(user.contents.groups.is_empty());
        assert!(user.contents.roles.is_empty());
    }

    let role = Role::named(format!("role-{}", server_name))
        .insert_into(admin)
        .await?;
    let group = PermissionGroup::named(format!("group-{}", server_name))
        .insert_into(admin)
        .await?;

    // Add the role and group.
    server.add_permission_group_to_user(user_id, &group).await?;
    server.add_role_to_user(user_id, &role).await?;

    // Test the results
    {
        let user = User::get(user_id, admin)
            .await
            .unwrap()
            .expect("user not found");
        assert_eq!(user.contents.groups, vec![group.header.id]);
        assert_eq!(user.contents.roles, vec![role.header.id]);
    }

    // Add the same things again (should not do anything). With names this time.
    server
        .add_permission_group_to_user(&username, &group)
        .await?;
    server.add_role_to_user(&username, &role).await?;
    {
        // TODO this is what's failing.
        let user = User::load(&username, admin)
            .await
            .unwrap()
            .expect("user not found");
        assert_eq!(user.contents.groups, vec![group.header.id]);
        assert_eq!(user.contents.roles, vec![role.header.id]);
    }

    // Remove the group.
    server
        .remove_permission_group_from_user(user_id, &group)
        .await?;
    server.remove_role_from_user(user_id, &role).await?;
    {
        let user = User::get(user_id, admin)
            .await
            .unwrap()
            .expect("user not found");
        assert!(user.contents.groups.is_empty());
        assert!(user.contents.roles.is_empty());
    }

    // Removing again shouldn't cause an error.
    server
        .remove_permission_group_from_user(user_id, &group)
        .await?;
    server.remove_role_from_user(user_id, &role).await?;

    Ok(())
}

/// Defines the `KeyValue` test suite
#[macro_export]
macro_rules! define_kv_test_suite {
    ($harness:ident) => {
        #[tokio::test]
        async fn basic_kv_test() -> anyhow::Result<()> {
            use $crate::keyvalue::{KeyStatus, KeyValue};
            let harness = $harness::new($crate::test_util::HarnessTest::KvBasic).await?;
            let db = harness.connect().await?;
            assert_eq!(
                db.set_key("akey", &String::from("avalue")).await?,
                KeyStatus::Inserted
            );
            assert_eq!(
                db.get_key("akey").into().await?,
                Some(String::from("avalue"))
            );
            assert_eq!(
                db.set_key("akey", &String::from("new_value"))
                    .returning_previous_as()
                    .await?,
                Some(String::from("avalue"))
            );
            assert_eq!(
                db.get_key("akey").into().await?,
                Some(String::from("new_value"))
            );
            assert_eq!(
                db.get_key("akey").and_delete().into().await?,
                Some(String::from("new_value"))
            );
            assert_eq!(db.get_key("akey").await?, None);
            assert_eq!(
                db.set_key("akey", &String::from("new_value"))
                    .returning_previous()
                    .await?,
                None
            );
            assert_eq!(db.delete_key("akey").await?, KeyStatus::Deleted);
            assert_eq!(db.delete_key("akey").await?, KeyStatus::NotChanged);

            harness.shutdown().await?;

            Ok(())
        }

        #[tokio::test]
        async fn kv_set_tests() -> anyhow::Result<()> {
            use $crate::keyvalue::{KeyStatus, KeyValue};
            let harness = $harness::new($crate::test_util::HarnessTest::KvSet).await?;
            let db = harness.connect().await?;
            let kv = db.with_key_namespace("set");

            assert_eq!(
                kv.set_key("a", &0_u32).only_if_exists().await?,
                KeyStatus::NotChanged
            );
            assert_eq!(
                kv.set_key("a", &0_u32).only_if_vacant().await?,
                KeyStatus::Inserted
            );
            assert_eq!(
                kv.set_key("a", &1_u32).only_if_vacant().await?,
                KeyStatus::NotChanged
            );
            assert_eq!(
                kv.set_key("a", &2_u32).only_if_exists().await?,
                KeyStatus::Updated,
            );
            assert_eq!(
                kv.set_key("a", &3_u32).returning_previous_as().await?,
                Some(2_u32),
            );

            harness.shutdown().await?;

            Ok(())
        }

        #[tokio::test]
        async fn kv_increment_decrement_tests() -> anyhow::Result<()> {
            use $crate::keyvalue::{KeyStatus, KeyValue};
            let harness =
                $harness::new($crate::test_util::HarnessTest::KvIncrementDecrement).await?;
            let db = harness.connect().await?;
            let kv = db.with_key_namespace("increment_decrement");

            // Empty keys should be equal to 0
            assert_eq!(kv.increment_key_by("i64", 1_i64).await?, 1_i64);
            assert_eq!(kv.get_key("i64").into_i64().await?, Some(1_i64));
            assert_eq!(kv.increment_key_by("u64", 1_u64).await?, 1_u64);
            $crate::assert_f64_eq!(kv.increment_key_by("f64", 1_f64).await?, 1_f64);

            // Test float incrementing/decrementing an existing value
            $crate::assert_f64_eq!(kv.increment_key_by("f64", 1_f64).await?, 2_f64);
            $crate::assert_f64_eq!(kv.decrement_key_by("f64", 2_f64).await?, 0_f64);

            // Empty keys should be equal to 0
            assert_eq!(kv.decrement_key_by("i64_2", 1_i64).await?, -1_i64);
            assert_eq!(
                kv.decrement_key_by("u64_2", 42_u64)
                    .allow_overflow()
                    .await?,
                u64::MAX - 41
            );
            assert_eq!(kv.decrement_key_by("u64_3", 42_u64).await?, u64::MIN);
            $crate::assert_f64_eq!(kv.decrement_key_by("f64_2", 1_f64).await?, -1_f64);

            // Test decrement wrapping with overflow
            kv.set_numeric_key("i64", i64::MIN).await?;
            assert_eq!(
                kv.decrement_key_by("i64", 1_i64).allow_overflow().await?,
                i64::MAX
            );
            assert_eq!(
                kv.decrement_key_by("u64", 2_u64).allow_overflow().await?,
                u64::MAX
            );

            // Test increment wrapping with overflow
            assert_eq!(
                kv.increment_key_by("i64", 1_i64).allow_overflow().await?,
                i64::MIN
            );
            assert_eq!(
                kv.increment_key_by("u64", 1_u64).allow_overflow().await?,
                u64::MIN
            );

            // Test saturating increments.
            kv.set_numeric_key("i64", i64::MAX - 1).await?;
            kv.set_numeric_key("u64", u64::MAX - 1).await?;
            assert_eq!(kv.increment_key_by("i64", 2_i64).await?, i64::MAX);
            assert_eq!(kv.increment_key_by("u64", 2_u64).await?, u64::MAX);

            // Test saturating decrements.
            kv.set_numeric_key("i64", i64::MIN + 1).await?;
            kv.set_numeric_key("u64", u64::MIN + 1).await?;
            assert_eq!(kv.decrement_key_by("i64", 2_i64).await?, i64::MIN);
            assert_eq!(kv.decrement_key_by("u64", 2_u64).await?, u64::MIN);

            // Test numerical conversion safety using get
            {
                // For i64 -> f64, the limit is 2^52 + 1 in either posive or
                // negative directions.
                kv.set_numeric_key("i64", (2_i64.pow(f64::MANTISSA_DIGITS)))
                    .await?;
                $crate::assert_f64_eq!(
                    kv.get_key("i64").into_f64().await?.unwrap(),
                    9_007_199_254_740_992_f64
                );
                kv.set_numeric_key("i64", -(2_i64.pow(f64::MANTISSA_DIGITS)))
                    .await?;
                $crate::assert_f64_eq!(
                    kv.get_key("i64").into_f64().await?.unwrap(),
                    -9_007_199_254_740_992_f64
                );

                kv.set_numeric_key("i64", (2_i64.pow(f64::MANTISSA_DIGITS) + 1))
                    .await?;
                assert!(matches!(kv.get_key("i64").into_f64().await, Err(_)));
                $crate::assert_f64_eq!(
                    kv.get_key("i64").into_f64_lossy().await?.unwrap(),
                    9_007_199_254_740_993_f64
                );
                kv.set_numeric_key("i64", -(2_i64.pow(f64::MANTISSA_DIGITS) + 1))
                    .await?;
                assert!(matches!(kv.get_key("i64").into_f64().await, Err(_)));
                $crate::assert_f64_eq!(
                    kv.get_key("i64").into_f64_lossy().await?.unwrap(),
                    -9_007_199_254_740_993_f64
                );

                // For i64 -> u64, the only limit is sign.
                kv.set_numeric_key("i64", -1_i64).await?;
                assert!(matches!(kv.get_key("i64").into_u64().await, Err(_)));
                assert_eq!(
                    kv.get_key("i64").into_u64_lossy(true).await?.unwrap(),
                    0_u64
                );
                assert_eq!(
                    kv.get_key("i64").into_u64_lossy(false).await?.unwrap(),
                    u64::MAX
                );

                // For f64 -> i64, the limit is fractional numbers. Saturating isn't tested in this conversion path.
                kv.set_numeric_key("f64", 1.1_f64).await?;
                assert!(matches!(kv.get_key("f64").into_i64().await, Err(_)));
                assert_eq!(
                    kv.get_key("f64").into_i64_lossy(false).await?.unwrap(),
                    1_i64
                );
                kv.set_numeric_key("f64", -1.1_f64).await?;
                assert!(matches!(kv.get_key("f64").into_i64().await, Err(_)));
                assert_eq!(
                    kv.get_key("f64").into_i64_lossy(false).await?.unwrap(),
                    -1_i64
                );

                // For f64 -> u64, the limit is fractional numbers or negative numbers. Saturating isn't tested in this conversion path.
                kv.set_numeric_key("f64", 1.1_f64).await?;
                assert!(matches!(kv.get_key("f64").into_u64().await, Err(_)));
                assert_eq!(
                    kv.get_key("f64").into_u64_lossy(false).await?.unwrap(),
                    1_u64
                );
                kv.set_numeric_key("f64", -1.1_f64).await?;
                assert!(matches!(kv.get_key("f64").into_u64().await, Err(_)));
                assert_eq!(
                    kv.get_key("f64").into_u64_lossy(false).await?.unwrap(),
                    0_u64
                );

                // For u64 -> i64, the limit is > i64::MAX
                kv.set_numeric_key("u64", i64::MAX as u64 + 1).await?;
                assert!(matches!(kv.get_key("u64").into_i64().await, Err(_)));
                assert_eq!(
                    kv.get_key("u64").into_i64_lossy(true).await?.unwrap(),
                    i64::MAX
                );
                assert_eq!(
                    kv.get_key("u64").into_i64_lossy(false).await?.unwrap(),
                    i64::MIN
                );
            }

            // Test that non-numeric keys won't be changed when attempting to incr/decr
            kv.set_key("non-numeric", &String::from("test")).await?;
            assert!(matches!(
                kv.increment_key_by("non-numeric", 1_i64).await,
                Err(_)
            ));
            assert!(matches!(
                kv.decrement_key_by("non-numeric", 1_i64).await,
                Err(_)
            ));
            assert_eq!(
                kv.get_key("non-numeric").into::<String>().await?.unwrap(),
                String::from("test")
            );

            harness.shutdown().await?;

            Ok(())
        }

        #[tokio::test]
        async fn kv_expiration_tests() -> anyhow::Result<()> {
            use std::time::Duration;

            use $crate::keyvalue::{KeyStatus, KeyValue};

            let harness = $harness::new($crate::test_util::HarnessTest::KvExpiration).await?;
            let db = harness.connect().await?;

            loop {
                let kv = db.with_key_namespace("expiration");

                kv.delete_key("a").await?;
                kv.delete_key("b").await?;

                // Test that the expiration is updated for key a, but not for key b.
                let timing = $crate::test_util::TimingTest::new(Duration::from_millis(500));
                let (r1, r2) = tokio::join!(
                    kv.set_key("a", &0_u32).expire_in(Duration::from_secs(2)),
                    kv.set_key("b", &0_u32).expire_in(Duration::from_secs(2))
                );
                if timing.elapsed() > Duration::from_millis(500) {
                    println!(
                        "Restarting test {}. Took too long {:?}",
                        line!(),
                        timing.elapsed(),
                    );
                    continue;
                }
                assert_eq!(r1?, KeyStatus::Inserted);
                assert_eq!(r2?, KeyStatus::Inserted);
                let (r1, r2) = tokio::join!(
                    kv.set_key("a", &1_u32).expire_in(Duration::from_secs(4)),
                    kv.set_key("b", &1_u32)
                        .expire_in(Duration::from_secs(100))
                        .keep_existing_expiration()
                );
                if timing.elapsed() > Duration::from_secs(1) {
                    println!(
                        "Restarting test {}. Took too long {:?}",
                        line!(),
                        timing.elapsed(),
                    );
                    continue;
                }

                assert_eq!(r1?, KeyStatus::Updated, "a wasn't an update");
                assert_eq!(r2?, KeyStatus::Updated, "b wasn't an update");

                let a = kv.get_key("a").into().await?;
                assert_eq!(a, Some(1u32), "a shouldn't have expired yet");

                // Before checking the value, make sure we haven't elapsed too
                // much time. If so, just restart the test.
                if !timing.wait_until(Duration::from_secs_f32(3.)).await {
                    println!(
                        "Restarting test {}. Took too long {:?}",
                        line!(),
                        timing.elapsed()
                    );
                    continue;
                }

                assert_eq!(kv.get_key("b").await?, None, "b never expired");

                timing.wait_until(Duration::from_secs_f32(5.)).await;
                assert_eq!(kv.get_key("a").await?, None, "a never expired");
                break;
            }
            harness.shutdown().await?;

            Ok(())
        }

        #[tokio::test]
        async fn delete_expire_tests() -> anyhow::Result<()> {
            use std::time::Duration;

            use $crate::keyvalue::{KeyStatus, KeyValue};

            let harness = $harness::new($crate::test_util::HarnessTest::KvDeleteExpire).await?;
            let db = harness.connect().await?;

            loop {
                let kv = db.with_key_namespace("delete_expire");

                kv.delete_key("a").await?;

                let timing = $crate::test_util::TimingTest::new(Duration::from_millis(500));

                // Create a key with an expiration. Delete the key. Set a new
                // value at that key with no expiration. Ensure it doesn't
                // expire.
                kv.set_key("a", &0_u32)
                    .expire_in(Duration::from_secs(2))
                    .await?;
                kv.delete_key("a").await?;
                kv.set_key("a", &1_u32).await?;
                if timing.elapsed() > Duration::from_secs(1) {
                    println!(
                        "Restarting test {}. Took too long {:?}",
                        line!(),
                        timing.elapsed(),
                    );
                    continue;
                }
                if !timing.wait_until(Duration::from_secs_f32(2.5)).await {
                    println!(
                        "Restarting test {}. Took too long {:?}",
                        line!(),
                        timing.elapsed()
                    );
                    continue;
                }

                assert_eq!(kv.get_key("a").into().await?, Some(1u32));

                break;
            }
            harness.shutdown().await?;

            Ok(())
        }

        #[tokio::test]
        async fn kv_transaction_tests() -> anyhow::Result<()> {
            use std::time::Duration;

            use $crate::{
                connection::Connection,
                keyvalue::{KeyStatus, KeyValue},
            };
            let harness = $harness::new($crate::test_util::HarnessTest::KvTransactions).await?;
            let db = harness.connect().await?;
            // Generate several transactions that we can validate
            db.set_key("expires", &0_u32)
                .expire_in(Duration::from_secs(1))
                .await?;
            db.set_key("akey", &String::from("avalue")).await?;
            db.get_key("akey").and_delete().await?;
            db.set_numeric_key("nkey", 0_u64).await?;
            db.increment_key_by("nkey", 1_u64).await?;
            db.delete_key("nkey").await?;
            // Ensure this doesn't generate a transaction.
            db.delete_key("nkey").await?;

            tokio::time::sleep(Duration::from_secs(2)).await;

            let transactions = Connection::list_executed_transactions(&db, None, None).await?;
            let deleted_keys = transactions
                .iter()
                .filter_map(|tx| tx.changes.keys())
                .flatten()
                .filter(|changed_key| changed_key.deleted)
                .count();
            assert_eq!(deleted_keys, 3);
            let akey_changes = transactions
                .iter()
                .filter_map(|tx| tx.changes.keys())
                .flatten()
                .filter(|changed_key| changed_key.key == "akey")
                .count();
            assert_eq!(akey_changes, 2);
            let nkey_changes = transactions
                .iter()
                .filter_map(|tx| tx.changes.keys())
                .flatten()
                .filter(|changed_key| changed_key.key == "nkey")
                .count();
            assert_eq!(nkey_changes, 3);

            harness.shutdown().await?;

            Ok(())
        }
    };
}

pub struct TimingTest {
    tolerance: Duration,
    start: Instant,
}

impl TimingTest {
    #[must_use]
    pub fn new(tolerance: Duration) -> Self {
        Self {
            tolerance,
            start: Instant::now(),
        }
    }

    pub async fn wait_until(&self, absolute_duration: Duration) -> bool {
        let target = self.start + absolute_duration;
        let now = Instant::now();
        if now > target {
            let amount_past = now - target;

            // Return false if we're beyond the tolerance given
            amount_past < self.tolerance
        } else {
            tokio::time::sleep_until(target.into()).await;
            true
        }
    }

    #[must_use]
    pub fn elapsed(&self) -> Duration {
        Instant::now()
            .checked_duration_since(self.start)
            .unwrap_or_default()
    }
}

pub async fn basic_server_connection_tests<C: StorageConnection>(
    server: C,
    newdb_name: &str,
) -> anyhow::Result<()> {
    let mut schemas = server.list_available_schemas().await?;
    schemas.sort();
    assert!(schemas.contains(&Basic::schema_name()?));
    assert!(schemas.contains(&SchemaName::new("khonsulabs", "bonsaidb-admin")?));

    let databases = server.list_databases().await?;
    assert!(databases.iter().any(|db| db.name == "tests"));

    server.create_database::<Basic>(newdb_name, false).await?;
    server.delete_database(newdb_name).await?;

    assert!(matches!(
        server.delete_database(newdb_name).await,
        Err(Error::DatabaseNotFound(_))
    ));

    assert!(matches!(
        server.create_database::<Basic>("tests", false).await,
        Err(Error::DatabaseNameAlreadyTaken(_))
    ));

    assert!(matches!(
        server.create_database::<Basic>("tests", true).await,
        Ok(_)
    ));

    assert!(matches!(
        server.create_database::<Basic>("|invalidname", false).await,
        Err(Error::InvalidDatabaseName(_))
    ));

    assert!(matches!(
        server
            .create_database::<UnassociatedCollection>(newdb_name, false)
            .await,
        Err(Error::SchemaNotRegistered(_))
    ));

    Ok(())
}
