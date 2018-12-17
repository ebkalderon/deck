use std::fmt::Debug;

pub mod backends;

mod id;

pub trait BinaryCache: Debug {
    fn query();
}
