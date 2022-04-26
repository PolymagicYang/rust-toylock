//! A LinkedList-like locks queue which is similar to CLH Lock but more efficient.
use std::{cell::UnsafeCell, sync::atomic::{AtomicPtr, AtomicBool, Ordering}, marker, ops::Deref};
use crate::{Lock, Guard};

#[derive(Debug, Default)]
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
#[derive(Debug, Default)]
pub struct CmsLock<T: Send + Sync> {
    node: AtomicPtr<Option<Node>>,
    data: UnsafeCell<T>,
}

#[derive(Debug)]
pub struct LockGuard<'a, T: 'a + Send + Sync> {
    data: *mut T,    
    lock: AtomicPtr<Option<Node>>,
    _marker: marker::PhantomData<&'a T>,
}

impl<'a, T> Drop for LockGuard<'a, T>
where 
    T: Send + Sync
{
    fn drop(&mut self) {
        self.unlock();
    } 
}

impl<'a, T: Send + Sync + 'a> Lock<'a, T> for CmsLock<T> {
    type G = LockGuard<'a, T>;

    fn lock(&'a self) -> Self::G {
        let curr = Box::into_raw(Box::new(Some(Node::new())));
        let prev = unsafe { 
            // AcqRel: make sure the load and store is updated.
            Box::from_raw(self.node.swap(curr, Ordering::AcqRel)) 
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
            // backoff.sniff().
        };

        LockGuard {
            data: self.data.get(), 
            lock: AtomicPtr::new(Box::into_raw(curr)),
            _marker: Default::default(),
        }
    }
}

impl<'a, T> Deref for LockGuard<'a, T>
where
    T: Send + Sync
{
    type Target = T;
    fn deref(&self) -> &Self::Target {
        unsafe { &*self.data }
    }
}

impl<'a, T: Send + Sync> Guard<T> for LockGuard<'a, T> {
    fn unlock(&self) {
        let curr_ptr = self.lock.load(Ordering::Acquire);
        let curr = unsafe {
            &*curr_ptr
        }.as_ref().unwrap();
       
        let next = unsafe {
            &*curr.next.load(Ordering::Acquire)
        };
        let new_node = Box::into_raw(Box::new(None));
        
        loop {
            match next {
                Some(next_node) => {
                    next_node.is_locked.store(false, Ordering::Release);
                    unsafe { Box::from_raw(curr_ptr) };
                    return
                }, 
                None => {
                    if self.lock.compare_exchange(curr_ptr, new_node, Ordering::Release, Ordering::Relaxed).is_ok() {
                        unsafe { Box::from_raw(curr_ptr) };
                        return 
                    }
                } 
            }
        }
        
        // drops the current node.
    }

}
