pub mod core;
pub mod integration;
pub mod prelude;
pub mod util;

use crate::core::ProjectRegistry;
use crate::prelude::*;
use std::sync::Arc;
use tokio::sync::Mutex;

errors! {
    pub enum AppError {
        Core => core::ProjectError,
    } {}
    pub type AppResult<T>;
}

#[tokio::main]
async fn main() {
    let projects = SharedProjects::new(ProjectRegistry::new());
}

#[derive(Clone)]
pub struct SharedProjects(Arc<Mutex<(ProjectRegistry, u64)>>);
impl SharedProjects {
    fn new(projects: ProjectRegistry) -> Self {
        Self(Arc::new(Mutex::new((projects, 0))))
    }

    pub async fn read<R>(&self, f: impl FnOnce(&ProjectRegistry) -> R) -> R {
        let inner = self.0.lock().await;
        let (registry, _) = &*inner;
        f(registry)
    }

    pub async fn read_with_revision<R>(&self, f: impl FnOnce(&ProjectRegistry) -> R) -> (R, u64) {
        let inner = self.0.lock().await;
        let (registry, revision) = &*inner;
        (f(registry), *revision)
    }

    pub async fn modify<R>(&self, f: impl FnOnce(&mut ProjectRegistry) -> R) -> R {
        let mut inner = self.0.lock().await;
        let (registry, revision) = &mut *inner;
        let result = f(registry);
        *revision += 1;
        result
    }
}
