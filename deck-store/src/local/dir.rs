pub use self::path::{ReadPath, WritePath};
pub use self::state::State;

use std::fmt::Debug;
use std::future::Future;
use std::pin::Pin;

use deck_core::FilesystemId;

mod path;
mod state;

// NOTE: All this noise has been to work fine with a simple `async fn`, with no need for associated
// types, this type alias, or `Pin<Box<_>>`. Replace _immediately_ once `async fn` in traits is
// stabilized in Rust.

pub type DirFuture<'a, T> = Pin<Box<dyn Future<Output = Result<T, ()>> + Send + 'a>>;

pub trait Directory: Debug + Send + Sync {
    type Id: FilesystemId;
    type Input: Send;
    type Output: Send;

    const NAME: &'static str;

    fn precompute_id<'a>(&'a self, input: &'a Self::Input) -> DirFuture<'a, Self::Id>;
    fn compute_id<'a>(&'a self, path: &'a ReadPath) -> DirFuture<'a, Self::Id>;
    fn read<'a>(&'a self, path: &'a ReadPath) -> DirFuture<'a, Option<Self::Output>>;
    fn write<'a>(
        &'a self,
        path: &'a mut WritePath,
        input: Self::Input,
    ) -> DirFuture<'a, Self::Output>;
}
