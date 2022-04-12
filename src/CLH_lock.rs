//! A linked-list like lock. 

use crate::{Lock, Guard};
/// A CLH lock consists of many nodes linearly linked together. 
///
/// Each working thread can hold one of the nodes to enable data & control synchronizations. 
/// The current thread has to wait for the previous thread to release the lock to execute the current task.
pub struct CLHLock {

}

pub struct LockGuard<'a, T: Send + Sync> {
    data: *mut T, 
    lock: &'a CLHLock,
}

impl<'a, T: Send + Sync> Guard for LockGuard<'a, T> {
    fn unlock(&self) {
        
    }
}

impl<'a, T: Send + Sync> Lock<'a, T> for CLHLock {
    type L = LockGuard<'a, T>;
    fn lock(&self) -> Self::L {

    }
}
