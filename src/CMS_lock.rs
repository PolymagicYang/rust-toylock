//! A LinkedList-like locks queue which is similar to CLH Lock but more efficient.
use std::{cell::UnsafeCell, sync::atomic::{AtomicPtr, AtomicBool, Ordering}, marker, ptr};
use backoff;
use crate::{Lock, Guard};

pub struct Node {
    is_locked: AtomicBool,
    next: AtomicPtr<Option<Node>>,    
}

impl Node {
    fn new() -> Node {
        let node_ptr = Box::into_raw(Box::new(None));
        Node { 
            is_locked: AtomicBool::new(true), 
            next: AtomicPtr::new(node_ptr),
        }
    }
}

/// A CMS Lock is similar to CLH_lock [crate::CLH_lock], but has some differencies.
/// 1. CMS lock is cache-friendly because every CMS lock's node releases itself instead of releasing previous nodes.
/// 2. CMS lock is more efficient in the NUMA system.
pub struct CmsLock<T: Send + Sync> {
    node: AtomicPtr<Option<Node>>,
    data: UnsafeCell<T>,
}

pub struct LockGuard<'a, T: 'a + Send + Sync> {
    data: *mut T,    
    lock: *mut Node,
    _lock_marker: marker::PhantomData<Node>,
    _data_marker: marker::PhantomData<&'a T>,
}

impl<'a, T> Drop for LockGuard<'a, T>
where 
    T: Send + Sync
{
    fn drop(&mut self) {
        self.unlock();
        unsafe {
            ptr::drop_in_place(self.lock);
        };
    } 
}

impl<'a, T: Send + Sync + 'a> Lock<'a, T> for CmsLock<T> {
    type L = LockGuard<'a, T>;

    fn lock(&self) -> Self::L {
        let curr = Box::into_raw(Box::new(Some(Node::new())));
        let prev = unsafe { 
            Box::from_raw(self.node.swap(curr, Ordering::Relaxed)) 
        };
        
        // put the current node into the context.
        prev
            .map(|ref node| {
                node.next.store(curr, Ordering::Release);
            })
            .unwrap_or(
                self.node.store(curr, Ordering::Release)
            );
       
        let curr = unsafe { Box::from_raw(curr) };
        if curr.as_ref().as_ref().unwrap().is_locked.load(Ordering::Acquire) {
            backoff::ExponentialBackoff::default();
        };

        LockGuard {
            data: self.data.get(), 
            lock: Box::into_raw(Box::new(curr.unwrap())),
            _data_marker: Default::default(),
            _lock_marker: Default::default(),
        }
    }
}

impl<'a, T: Send + Sync> Guard for LockGuard<'a, T> {
    fn unlock(&self) {
        let curr = unsafe { Box::from_raw(self.lock) };
    }
}
