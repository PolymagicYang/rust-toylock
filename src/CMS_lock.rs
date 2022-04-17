//! A LinkedList-like locks queue which is similar to CLH Lock but more efficient.
use std::{cell::UnsafeCell, sync::atomic::{AtomicPtr, AtomicBool}};

use crate::{Lock, Guard};

pub struct Node<T: Send + Sync> {
    is_locked: AtomicBool,
    curr_lock: AtomicPtr<T>,    
}

pub struct CMS_lock<T: Send + Sync> {
    node: UnsafeCell<Node<T>>,
}

pub struct LockGuard<'a, T: 'a + Send + Sync> {
    data: *mut T,    
    lock: &'a Node<T>,
}

impl<'a, T: Send + Sync + 'a> Lock<'a, T> for CMS_lock<T> {
    type L = LockGuard<'a, T>;

    fn lock(&self) -> Self::L {

    }
}

impl<'a, T: Send + Sync> Guard for LockGuard<'a, T> {
    fn unlock(&self) {
          
    }
}

