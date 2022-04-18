//! A linked-list like lock. 

use backoff;
use crate::{Lock, Guard};
use std::{sync::atomic::{ AtomicBool, AtomicPtr, Ordering }, cell::UnsafeCell, thread, ops::{Deref, DerefMut}, time::Duration};
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
pub struct CLHLock<T: Send + Sync> {
    prev: NodePtr,
    data: UnsafeCell<T>,
}

struct Node {
    is_locked: AtomicBool, 
}

type NodePtr = AtomicPtr<Node>;

impl Default for Node {
    fn default() -> Self {
        Node { is_locked: AtomicBool::new(true) }
    }    
}

impl Node {
    fn new() -> Self {
        Node { is_locked: AtomicBool::new(false) } 
    } 
}

pub struct LockGuard<'a, T: Send + Sync> {
    data: &'a mut T, 
    lock: *mut Node,
}

impl<'a, T: Send + Sync> LockGuard<'a, T> {
    fn new(data: &'a mut T, lock: *mut Node) -> Self {
        LockGuard {
            data,
            lock
        } 
    }
}

impl<'a, T: Send + Sync> Guard for LockGuard<'a, T> {
    fn unlock(&self) {
        let curr = unsafe { Box::from_raw(self.lock) };
        curr.is_locked.store(false, Ordering::Release);
        // Let the next node drop the current node.
        Box::into_raw(curr);
    }
}

impl<'a, T: 'a + Send + Sync> Lock<'a, T> for CLHLock<T> {
    type L = LockGuard<'a, T>;
    fn lock(&self) -> Self::L {
        let curr = Box::into_raw(Box::new(Node::default()));
        // Ordering relaxed is accepable, because read-modify-write is message adjancy operation.
        let prev_ptr = self.prev.swap(curr, Ordering::Relaxed);

        let prev = unsafe { Box::from_raw(prev_ptr) };
        while prev.is_locked.load(Ordering::Acquire) {
            // Make sure previous node's lock is released.
            // Ordering::Acquire is required after other threads release the lock and change the value in node.
            backoff::ExponentialBackoff::default(); 
        }
        
        // Let the current node cleanup the previous node.
        LockGuard::new(unsafe {&mut *self.data.get()}, curr)
    }
}

impl<T: Send + Sync> CLHLock<T> {
    fn new(data: T) -> Self {
        let node_ptr = Box::into_raw(Box::new(Node::new()));
        CLHLock { prev: AtomicPtr::new(node_ptr), data: UnsafeCell::new(data) }
    }
}

impl<'a, T: Send + Sync> Deref for LockGuard<'a, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        self.data 
    } 
}

impl<'a, T: Send + Sync> DerefMut for LockGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.data 
    } 
}

unsafe impl<T: Send + Sync> Send for CLHLock<T> {}
unsafe impl<T: Send + Sync> Sync for CLHLock<T> {}

#[test]
fn test() {
    let test: &'static CLHLock<Vec<i32>> = Box::leak(Box::new(CLHLock::new(vec![])));
	let joins: Vec<_> = (1..100)
		.map(|x| {
			thread::spawn(move || {
				let mut a = test.lock();
				a.push(x);
				a.unlock();	
			})
	}).collect();
    
    let _: Vec<_> = joins.into_iter().map(|x| {
        x.join().unwrap();
    }).collect();
	// done!
    // 
    println!("{:?}", *test.lock());
}

#[test]
fn test1() {
    let test: &'static CLHLock<Vec<i32>> = Box::leak(Box::new(CLHLock::new(vec![])));
	let _joins: Vec<_> = (1..100)
		.map(|x| {
			thread::spawn(move || {
				let mut a = test.lock();
				a.push(x);
				a.unlock();	
			})
	}).collect();
    
    thread::sleep(Duration::from_millis(100));
    // 
    println!("{:?}", *test.lock());
}

#[test]
fn test_2() {
    let a = thread::spawn(move || { 
        thread::park();
        println!("Hello!");
    });
    a.thread().unpark();
}
