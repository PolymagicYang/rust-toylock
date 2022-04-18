//! A LinkedList-like locks queue which is similar to CLH Lock but more efficient.
use std::{cell::UnsafeCell, sync::atomic::{AtomicPtr, AtomicBool, Ordering}};
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

pub struct LockGuard<T: Send + Sync> {
    data: *mut T,    
    lock: *mut Node,
}

impl<'a, T: Send + Sync + 'a> Lock<'a, T> for CmsLock<T> {
    type L = LockGuard<T>;

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
        }
    }
}

impl<'a, T: Send + Sync> Guard for LockGuard<T> {
    fn unlock(&self) {

    }
}

