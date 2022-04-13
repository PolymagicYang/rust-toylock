//! Ordering matters lock.  

// Ticket Lock -> Sequentiall consistent fetch the lock in a specific ordering.
// thread1 -> thread2 -> thread3 -> ... (1 .. n is the ordering)
// Two states: 1. current => for the current ticket number.
//             2. ticket_num => for the ticket num.
//     when current == ticket_num => thread holds the lock
//     unlock => (fetch and increment) the current.

use std::{sync::atomic::{AtomicUsize, Ordering}, cell::UnsafeCell, ops::DerefMut, thread};
use core::ops::Deref;

unsafe impl<T: Send + Sync> Send for TicketLock<T> {}
unsafe impl<T: Send + Sync> Sync for TicketLock<T> {}

struct Ticket {
	current: AtomicUsize,
	ticket_num: AtomicUsize,
}

// T must be bouned by Send and Sync.
// cause: T can be sent from one thread to other threads.
//        T can be accesssed by many threads at the same time.
// unsafecell: 
//     	     1. immutability => can be shared.
//           2. internal mutability => mutability
pub struct TicketLock<T: Send + Sync> {
	data: UnsafeCell<T>,
	lock: Ticket,
}

pub struct LockGuard<'a, T> {
	// raw pointer.
	data: *mut T,
	lock: &'a Ticket, 
}

impl<'a, T: Send + Sync> LockGuard<'a, T> {
	pub fn unlock(&self) {
		// invariance: only one thread holds the lock.
		// Release: make all the varables modified be visible from other threads.
		// or: updates the per-message view.
		self.lock.current.fetch_add(1, Ordering::Release);
	}
}

impl<'a, T: Send + Sync> TicketLock<T> {
	pub fn lock(&'a self) -> LockGuard<'a, T> {
		// fetch and add is message view (adjacent message), Ordering doesn't matter.
		let curr = self.lock.ticket_num.fetch_add(1, Ordering::Relaxed);

		while curr != self.lock.current.load(Ordering::Acquire) {
			todo!("sleep or backoff")	
		};

		LockGuard { 
			data: self.data.get(), 
			lock: &self.lock, 
		}
	}

}

impl<'a, T: Sync + Send> Deref for LockGuard<'a, T> {
	type Target = T;
	// hold lock => get guard => data safe.
	fn deref(&self) ->  &Self::Target {
		unsafe { &*self.data }
	}
}

impl<'a, T: Sync + Send> DerefMut for LockGuard<'a, T> {
	fn deref_mut(&mut self) -> &mut Self::Target {
		unsafe { &mut *self.data }
	}
}

impl Ticket {
	fn new() -> Self {
		Ticket { 
			current: AtomicUsize::new(0),
			ticket_num: AtomicUsize::new(0)
		}
	}
}

impl<T: Send + Sync> TicketLock<T> {
	pub fn new(data: T) -> Self {
		TicketLock { 
			data: UnsafeCell::new(data), 
			lock: Ticket::new() 
		}
	}

}

#[test]
fn test() {
	// leak: get static lifetime reference.
	let test: &'static TicketLock<Vec<i32>> = Box::leak(Box::new(TicketLock::new(vec![])));
	let _joins: Vec<_> = (1..100)
		.map(|x| {
			thread::spawn(move || {
				let mut a = test.lock();
				a.push(x);
				a.unlock();	
			})
	})
	.map(|x| {
		x.join()
	}).collect();
    
	println!("{:?}", *test.lock());
	// done!
}