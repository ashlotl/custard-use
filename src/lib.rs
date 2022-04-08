#![feature(arbitrary_self_types)]
#![feature(negative_impls)]
#![feature(path_try_exists)]
#![feature(try_trait_v2)]

pub mod composition;
pub mod concurrency;
pub mod dylib_management;
pub mod errors;
pub mod identify;
pub mod user_types;
pub mod utils;

pub mod custard_instance;
pub mod instance_control_flow;
