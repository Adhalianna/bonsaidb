use async_trait::async_trait;

use super::{KeyOperation, Kv, Output};
use crate::Error;

/// A namespaced key-value store. All operations performed with this will be
/// separate from other namespaces.
pub struct Namespaced<'a, K> {
    namespace: String,
    kv: &'a K,
}

impl<'a, K> Namespaced<'a, K> {
    pub(crate) const fn new(namespace: String, kv: &'a K) -> Self {
        Self { namespace, kv }
    }
}

#[async_trait]
impl<'a, K> Kv for Namespaced<'a, K>
where
    K: Kv,
{
    async fn execute(&self, op: KeyOperation) -> Result<Output, Error> {
        self.kv.execute(op).await
    }

    fn namespace(&self) -> Option<&'_ str> {
        Some(&self.namespace)
    }

    fn with_namespace(&'_ self, namespace: &str) -> Namespaced<'_, Self>
    where
        Self: Sized,
    {
        Namespaced {
            namespace: format!("{}\u{0}{}", self.namespace, namespace),
            kv: self,
        }
    }
}
