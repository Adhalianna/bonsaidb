use pliantdb_core::schema::collection;
use serde::{Deserialize, Serialize};

use self::{integrity_scanner::IntegrityScan, mapper::Map};

#[derive(Serialize, Deserialize)]
pub struct ViewEntry {
    pub view_version: usize,
    pub mappings: Vec<EntryMapping>,
}

#[derive(Serialize, Deserialize)]
pub struct EntryMapping {
    pub source: u64,
    pub value: Vec<u8>,
}

pub mod integrity_scanner;
pub mod mapper;

pub fn view_entries_tree_name(collection: &collection::Id, view_name: &str) -> String {
    format!("{}::{}", collection.0, view_name)
}

/// Used to store Document ID -> Key mappings, so that when a document is updated, we can remove the old entry.
pub fn view_document_map_tree_name(collection: &collection::Id, view_name: &str) -> String {
    format!("{}::{}::document-map", collection.0, view_name)
}

pub fn view_invalidated_docs_tree_name(collection: &collection::Id, view_name: &str) -> String {
    format!("{}::{}::invalidated", collection.0, view_name)
}

pub fn view_omitted_docs_tree_name(collection: &collection::Id, view_name: &str) -> String {
    format!("{}::{}::omitted", collection.0, view_name)
}

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub enum Task {
    IntegrityScan(IntegrityScan),
    ViewMap(Map),
}
