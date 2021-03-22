use std::borrow::Cow;

use serde::{Deserialize, Serialize};

use crate::document::Document;

/// types for defining a `Map` within a `View`
pub mod map;
pub use map::{Key, Map};

/// errors that arise when interacting with views
#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// an error occurred while serializing or deserializing
    #[error("error deserializing document {0}")]
    SerializationError(#[from] serde_cbor::Error),

    /// returned when
    #[error("reduce is unimplemented")]
    ReduceUnimplemented,
}

/// a type alias for the result of `View::map()`
pub type MapResult<'k, K = (), V = ()> = Result<Option<Map<'k, K, V>>, Error>;

/// a map/reduce powered indexing and aggregation schema
///
/// inspired by [`CouchDB`'s view system](https://docs.couchdb.org/en/stable/ddocs/views/index.html)
// TODO write our own view docs
pub trait View<'k> {
    /// the key for this view. If you're using ranged queries, this type must be
    /// meaningfully sortable when converted to bytes. Additionally, the
    /// conversion process to bytes must be done using a consistent endianness.
    type MapKey: Key<'k> + 'static;

    /// an associated type that can be stored with each entry in the view
    type MapValue: Serialize + for<'de> Deserialize<'de>;

    /// when implementing reduce(), this is the returned type
    type Reduce: Serialize + for<'de> Deserialize<'de>;

    /// the version of the view. Changing this value will cause indexes to be rebuilt.
    fn version() -> usize;

    /// the name of the view. Must be unique per collection.
    fn name() -> Cow<'static, str>;

    /// the map function for this view. This function is responsible for
    /// emitting entries for any documents that should be contained in this
    /// View. If None is returned, the View will not include the document.
    fn map(document: &Document<'_>) -> MapResult<'k, Self::MapKey, Self::MapValue>;

    /// the reduce function for this view. If `Err(Error::ReduceUnimplemented)`
    /// is returned, queries that ask for a reduce operation will return an
    /// error. See [`CouchDB`'s Reduce/Rereduce
    /// documentation](https://docs.couchdb.org/en/stable/ddocs/views/intro.html#reduce-rereduce)
    /// for the design this implementation will be inspired by
    #[allow(unused_variables)]
    fn reduce(
        mappings: &[Map<'k, Self::MapKey, Self::MapValue>],
        rereduce: bool,
    ) -> Result<Self::Reduce, Error> {
        Err(Error::ReduceUnimplemented)
    }
}

/// an enum representing either an owned value or a borrowed value. Functionally
/// equivalent to `std::borrow::Cow` except this type doesn't require the
/// wrapped type to implement `Clone`.
pub enum SerializableValue<'a, T: Serialize> {
    /// an owned value
    Owned(T),
    /// a borrowed value
    Borrowed(&'a T),
}

impl<'a, T> From<&'a T> for SerializableValue<'a, T>
where
    T: Serialize,
{
    fn from(other: &'a T) -> SerializableValue<'a, T> {
        SerializableValue::Borrowed(other)
    }
}

impl<'a, T> AsRef<T> for SerializableValue<'a, T>
where
    T: Serialize,
{
    fn as_ref(&self) -> &T {
        match self {
            Self::Owned(value) => value,
            Self::Borrowed(value) => value,
        }
    }
}

pub(crate) trait Serialized<'k> {
    fn name() -> Cow<'static, str>;
    fn map(document: &Document<'_>) -> Result<Option<map::Serialized<'k>>, Error>;
}

impl<'k, T> Serialized<'k> for T
where
    T: View<'k>,
    <T as View<'k>>::MapKey: 'static,
{
    fn name() -> Cow<'static, str> {
        Self::name()
    }

    fn map(document: &Document<'_>) -> Result<Option<map::Serialized<'k>>, Error> {
        let map = Self::map(document)?;

        match map {
            Some(map) => Ok(Some(map::Serialized {
                source: map.source,
                key: map.key.into_endian_bytes(),
                value: serde_cbor::value::to_value(&map.value)?,
            })),
            None => Ok(None),
        }
    }
}
