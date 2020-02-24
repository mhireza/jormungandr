use crate::blockchain::Ref;
use futures03::{
    future::FutureExt,
    stream::{futures_unordered::FuturesUnordered, StreamExt},
};
use std::{convert::Infallible, iter::FromIterator, sync::Arc};
use tokio::{prelude::*, sync::lock::Lock};
use tokio02::sync::RwLock;
use tokio_compat::prelude::*;

#[derive(Clone)]
pub struct Branches {
    inner: Arc<RwLock<BranchesData>>,
}

struct BranchesData {
    branches: Vec<Branch>,
}

#[derive(Clone)]
pub struct Branch {
    inner: Lock<BranchData>,
}

/// the data that is contained in a branch
struct BranchData {
    /// reference to the block where the branch points to
    reference: Arc<Ref>,

    last_updated: std::time::SystemTime,
}

impl Branches {
    pub fn new() -> Self {
        Branches {
            inner: Arc::new(RwLock::new(BranchesData {
                branches: Vec::new(),
            })),
        }
    }

    pub async fn add(&mut self, branch: Branch) {
        let mut guard = self.inner.write().await;
        guard.add(branch);
    }

    pub async fn apply_or_create(&mut self, candidate: Arc<Ref>) -> Branch {
        let mut guard = self.inner.write().await;
        if let Some(branch) = guard.apply(Arc::clone(&candidate)).await {
            branch
        } else {
            let branch = Branch::new(candidate);
            guard.add(branch.clone());
            branch
        }
    }

    pub async fn branches(&self) -> Vec<Arc<Ref>> {
        let guard = self.inner.read().await;
        guard.branches().await
    }
}

impl BranchesData {
    fn add(&mut self, branch: Branch) {
        self.branches.push(branch)
    }

    pub async fn apply(&mut self, candidate: Arc<Ref>) -> Option<Branch> {
        let branches_futures = self
            .branches
            .iter_mut()
            .map(|branch| branch.continue_with(Arc::clone(&candidate)));

        FuturesUnordered::from_iter(branches_futures)
            .filter_map(|updated| async move { updated }.boxed())
            .into_future()
            .map(|(v, _)| v)
            .await
    }

    pub async fn branches(&self) -> Vec<Arc<Ref>> {
        let branches_futures = self.branches.iter().map(|b| b.get_ref_std());

        FuturesUnordered::from_iter(branches_futures)
            .collect()
            .await
    }
}

impl Branch {
    pub fn new(reference: Arc<Ref>) -> Self {
        Branch {
            inner: Lock::new(BranchData::new(reference)),
        }
    }

    pub async fn get_ref_std(&self) -> Arc<Ref> {
        let mut branch = self.inner.clone();
        let r: Result<_, ()> = future::poll_fn(move || Ok(branch.poll_lock()))
            .map(|guard| guard.reference().clone())
            .compat()
            .await;
        r.unwrap()
    }

    pub async fn update_ref_std(&mut self, new_ref: Arc<Ref>) -> Arc<Ref> {
        let mut branch = self.inner.clone();
        let r: Result<_, ()> = future::poll_fn(move || Ok(branch.poll_lock()))
            .map(move |mut guard| guard.update(new_ref))
            .compat()
            .await;
        r.unwrap()
    }

    pub fn get_ref<E>(&self) -> impl Future<Item = Arc<Ref>, Error = E> {
        let mut branch = self.inner.clone();
        future::poll_fn(move || Ok(branch.poll_lock())).map(|guard| guard.reference().clone())
    }

    pub fn update_ref(
        &mut self,
        new_ref: Arc<Ref>,
    ) -> impl Future<Item = Arc<Ref>, Error = Infallible> {
        let mut branch = self.inner.clone();
        future::poll_fn(move || Ok(branch.poll_lock())).map(move |mut guard| guard.update(new_ref))
    }

    async fn continue_with(&mut self, candidate: Arc<Ref>) -> Option<Self> {
        let clone_branch = self.clone();
        let mut branch = self.inner.clone();
        let r: Result<_, ()> = future::poll_fn(move || Ok(branch.poll_lock()))
            .map(move |mut guard| guard.continue_with(candidate))
            .map(move |r| if r { Some(clone_branch) } else { None })
            .compat()
            .await;
        r.unwrap()
    }
}

impl BranchData {
    /// create the branch data with the current `last_updated` to
    /// the current time this function was called
    fn new(reference: Arc<Ref>) -> Self {
        BranchData {
            reference,
            last_updated: std::time::SystemTime::now(),
        }
    }

    fn update(&mut self, reference: Arc<Ref>) -> Arc<Ref> {
        let old_reference = std::mem::replace(&mut self.reference, reference);
        self.last_updated = std::time::SystemTime::now();

        old_reference
    }

    fn reference(&self) -> Arc<Ref> {
        Arc::clone(&self.reference)
    }

    fn continue_with(&mut self, candidate: Arc<Ref>) -> bool {
        if self.reference.hash() == candidate.block_parent_hash() {
            let _parent = self.update(candidate);
            true
        } else {
            false
        }
    }
}
