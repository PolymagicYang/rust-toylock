//! A LinkedList-like locks queue which is similar to CLH Lock but more efficient.
use std::{cell::UnsafeCell, sync::atomic::{AtomicPtr, AtomicBool, Ordering}, marker, ptr};
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
    lock: &'a AtomicPtr<Option<Node>>,
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
    type L = LockGuard<'a, T>;

    fn lock(&'a self) -> Self::L {
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
            // backoff.sniff().
        };

        LockGuard {
            data: self.data.get(), 
            lock: &self.node,
            _marker: Default::default(),
        }
    }
}

impl<'a, T: Send + Sync> Guard for LockGuard<'a, T> {
    fn unlock(&self) {
        let curr = unsafe {
            &*self.lock.load(Ordering::Acquire)
        }.as_ref().unwrap();
       
        let next = unsafe {
            &*curr.next.load(Ordering::Acquire)
        };
        
        match next {
            Some(next_node) => todo!("drop current node and make next node free."),
            None => todo!("compare and exchange to mitigate the dirty read."),
        }
    }
}
