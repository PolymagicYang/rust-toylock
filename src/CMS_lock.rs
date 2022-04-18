//! A LinkedList-like locks queue which is similar to CLH Lock but more efficient.
use std::{cell::UnsafeCell, sync::atomic::{AtomicPtr, AtomicBool, Ordering}};
use backoff;
use crate::{Lock, Guard};

pub struct Node {
    is_locked: AtomicBool,
    next: Option<AtomicPtr<Node>>,    
}

impl Node {
    fn new() -> Self {
        Node { 
            is_locked: AtomicBool::new(true), 
            next: None,
        }
    }
}

/// A CMS Lock is similar to CLH_lock [crate::CLH_lock], but has some differencies.
/// 1. CMS lock is cache-friendly because every CMS lock's node releases itself instead of releasing previous nodes.
/// 2. CMS lock is more efficient in the NUMA system.
pub struct CMS_lock<T: Send + Sync> {
    node: AtomicPtr<Node>,
    data: UnsafeCell<T>,
}

pub struct LockGuard<T: Send + Sync> {
    data: *mut T,    
    lock: *mut Node,
}

impl<'a, T: Send + Sync + 'a> Lock<'a, T> for CMS_lock<T> {
    type L = LockGuard<T>;

    fn lock(&self) -> Self::L {
        let curr = Box::into_raw(Box::new(Node::new()));
        let mut prev = unsafe { 
            self.node.swap(curr, Ordering::Relaxed).as_mut().unwrap() 
        };
        
        // put the current node into the context.
        match &mut prev.next {
            Some(ptr) => {
                ptr.store(curr, Ordering::Release);
            }
            None => {
                prev.next = Some(AtomicPtr::new(curr));
            }
        }
       
        let curr = unsafe { Box::from_raw(curr) };
        while curr.is_locked.load(Ordering::Acquire) {
            backoff::ExponentialBackoff::default();   
        };

        LockGuard {
            data: self.data.get(), 
            lock: Box::into_raw(curr),
        }
    }
}

impl<'a, T: Send + Sync> Guard for LockGuard<T> {
    fn unlock(&self) {

    }
}

