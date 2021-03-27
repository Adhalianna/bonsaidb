use std::{fmt::Debug, sync::Arc};

use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

use crate::{
    task::{Handle, Id},
    Job, Keyed,
};

pub(crate) mod jobs;
mod managed_job;
pub(crate) use managed_job::ManagedJob;

#[cfg(test)]
mod tests;

/// A background jobs manager.
#[derive(Debug)]
pub struct Manager<Key = ()> {
    pub(crate) jobs: Arc<RwLock<jobs::Jobs<Key>>>,
}

impl<Key> Default for Manager<Key>
where
    Key: Clone + std::hash::Hash + Eq + Send + Sync + Debug + 'static,
{
    fn default() -> Self {
        Self {
            jobs: Arc::new(RwLock::new(jobs::Jobs::new())),
        }
    }
}

impl<Key> Clone for Manager<Key> {
    fn clone(&self) -> Self {
        Self {
            jobs: self.jobs.clone(),
        }
    }
}

impl<Key> Manager<Key>
where
    Key: Clone + std::hash::Hash + Eq + Send + Sync + Debug + 'static,
{
    /// Pushes a `job` into the queue. Pushing the same job definition twice
    /// will yield two tasks in the queue.
    pub async fn enqueue<J: Job + 'static>(&self, job: J) -> Handle<J::Output, Key> {
        let mut jobs = self.jobs.write().await;
        jobs.enqueue(job, None, self.clone())
    }

    /// Uses [`Keyed::key`] to ensure no other job with the same `key` is
    /// currently running. If another job is already running that matches, a
    /// clone of that [`Handle`] will be returned. When the job finishes, all
    /// [`Handle`] clones will be notified with a copy of the result.
    pub async fn lookup_or_enqueue<J: Keyed<Key>>(
        &self,
        job: J,
    ) -> Handle<<J as Job>::Output, Key> {
        let mut jobs = self.jobs.write().await;
        jobs.lookup_or_enqueue(job, self.clone())
    }

    async fn job_completed<T: Serialize + for<'de> Deserialize<'de> + Send + Sync + 'static>(
        &self,
        id: Id,
        key: Option<&Key>,
        result: Result<T, anyhow::Error>,
    ) {
        let mut jobs = self.jobs.write().await;
        jobs.job_completed(id, key, result).await
    }

    /// Spawns a worker. In general, you shouldn't need to call this function
    /// directly.
    pub fn spawn_worker(&self) {
        let manager = self.clone();
        tokio::spawn(async move { manager.execute_jobs().await });
    }

    async fn execute_jobs(&self) {
        let receiver = {
            let jobs = self.jobs.read().await;
            jobs.queue()
        };
        while let Ok(mut job) = receiver.recv_async().await {
            job.execute().await
        }
    }
}
