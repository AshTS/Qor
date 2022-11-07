/// Object for syncronizing threads by allowing a signal to be sent from one thread to another
pub trait Semaphore: Send + Sync + Sized {
    /// Read the state of the semaphore, returning the result of the semaphore, and the semaphore in an option, to specify if the semaphore is still valid if the function returns true, the semaphore has been triggered
    fn read(self) -> (bool, Option<Self>);
}

pub mod signal;
pub use signal::*;

pub mod update;
pub use update::*;