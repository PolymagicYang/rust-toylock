//! A linked-list like lock. 

use crate::{Lock, Guard};
use std::sync::atomic::AtomicBool;
/// A CLH lock consists of many nodes linearly linked together. 
///
/// Each working thread can hold one of the nodes to enable data & control synchronizations. 
/// The current thread has to wait for the previous thread to release the lock to execute the current task.
/// Suppose we have tow threads access the CLHLock, all execution processes illustrated below:
///
/// 1. Initially, the CLHLock holds a node that contains a value that indicates an unlocked state (AtomicBool::False).
/// 2. The first thread swaps the current node with the newly generated node, which indicates a locked state. 
/// After swapping the nodes, the thread creates a new pointer to point to the previous node and checks: whether the previous node is locked. 
/// If the previous node is unlocked, it keeps executing its job; if not, it tries again until the previous node shows unlocked.
/// 3. When the first thread executes its task, the thread 2 gets the CLHLock and does the same job with step2. 
/// Thread 2 will see that the previous node shows locked because thread one doesn't complete its job.
/// 4. After threads 1 complete its job, thread 2 can start its job because the thread 1 will change its node to unlocked, which
/// is the previous node of thread 2. From that, the threads hold the following nodes can get the lock sequentially.
pub struct CLHLock {
    
}

struct Token {
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
