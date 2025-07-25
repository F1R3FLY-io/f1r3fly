use std::sync::Arc;
use tokio::sync::RwLock;

use super::engine::{noop, Engine};
use crate::rust::errors::CasperError;

/// EngineCell is a concurrency-safe mutable container for the current Engine instance.
///
/// This is the Rust equivalent of Scala's Cell[F, Engine[F]] (see EngineCell.scala).
/// It provides async operations that match the Scala F[_] monadic interface.
///
/// Usage:
///   let engine_cell = EngineCell::init().await?;
///   let engine = engine_cell.read().await?;
///   engine_cell.set(Box::new(MyEngine::new(...))).await?;
///
/// This implementation provides 1:1 API compatibility with the Scala EngineCell.
pub struct EngineCell {
    inner: Arc<RwLock<Box<dyn Engine>>>,
}

impl EngineCell {
    /// Initialize EngineCell with NoopEngine (equivalent to Cell.mvarCell[F, Engine[F]](Engine.noop))
    /// Returns Future<Result<EngineCell, CasperError>> to match Scala's F[EngineCell[F]]
    pub async fn init() -> Result<Self, CasperError> {
        let engine = Box::new(noop()?);
        Ok(EngineCell {
            inner: Arc::new(RwLock::new(engine)),
        })
    }

    /// Read the current engine (equivalent to Cell.read: F[Engine[F]])
    /// This is the most frequently used method in the Scala codebase
    pub async fn read(&self) -> Result<Box<dyn Engine>, CasperError> {
        // Clone the engine to match Scala's read behavior which doesn't hold locks
        let guard = self.inner.read().await;
        // Since we can't clone trait objects directly, we need to handle this differently
        // For now, we'll return a reference-counted clone of the inner Arc
        // This maintains the async behavior while providing access to the engine
        Ok(guard.clone_box())
    }

    /// Set the engine to a new instance (equivalent to Cell.set(s: Engine[F]): F[Unit])
    pub async fn set(&self, engine: Box<dyn Engine>) -> Result<(), CasperError> {
        let mut guard = self.inner.write().await;
        *guard = engine;
        Ok(())
    }

    /// Modify the engine with a pure function (equivalent to Cell.modify(f: Engine[F] => Engine[F]): F[Unit])
    pub async fn modify<F>(&self, f: F) -> Result<(), CasperError>
    where
        F: FnOnce(Box<dyn Engine>) -> Box<dyn Engine> + Send,
    {
        let mut guard = self.inner.write().await;
        let current_engine = std::mem::replace(&mut *guard, Box::new(noop()?));
        *guard = f(current_engine);
        Ok(())
    }

    /// Modify the engine with an async function (equivalent to Cell.flatModify(f: Engine[F] => F[Engine[F]]): F[Unit])
    pub async fn flat_modify<F, Fut>(&self, f: F) -> Result<(), CasperError>
    where
        F: FnOnce(Box<dyn Engine>) -> Fut + Send,
        Fut: std::future::Future<Output = Result<Box<dyn Engine>, CasperError>> + Send,
    {
        // Remove the current engine from the cell and replace it with a placeholder so
        // other readers will block until the new engine is written back. Keep a handle
        // to the original value so we can restore it if the async operation fails.
        let current_engine = {
            let mut guard = self.inner.write().await;
            std::mem::replace(&mut *guard, Box::new(noop()?))
        };

        // Execute the provided async function **without** holding the lock.
        let result = f(current_engine.clone_box()).await;

        // Re-acquire the lock and either commit the new engine or restore the old one,
        // matching Scala's bracket‐style resource handling (restore on failure).
        let mut guard = self.inner.write().await;
        match result {
            Ok(new_engine) => {
                *guard = new_engine;
                Ok(())
            }
            Err(e) => {
                *guard = current_engine; // restore previous state
                Err(e)
            }
        }
    }

    /// Read with a transformation function (equivalent to Cell.reads[A](f: Engine[F] => A): F[A])
    pub async fn reads<A, F>(&self, f: F) -> Result<A, CasperError>
    where
        F: FnOnce(&dyn Engine) -> A + Send,
        A: Send,
    {
        let guard = self.inner.read().await;
        Ok(f(guard.as_ref()))
    }
}

impl Clone for EngineCell {
    fn clone(&self) -> Self {
        EngineCell {
            inner: Arc::clone(&self.inner),
        }
    }
}
