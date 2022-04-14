//! A insufficient lock.

use std::{sync::{atomic::{ AtomicBool, Ordering }, Arc}, ops::{Deref, DerefMut}, cell::UnsafeCell};

// 自旋锁：持有锁的线程不断查看是否有其他线程在使用锁。

// 因为数据本身是可传递和同步的，所以锁也应该是可传递和可同步的。
unsafe impl<T: Send + Sync> Send for SpinLock<T> {}
unsafe impl<T: Send + Sync> Sync for SpinLock<T> {}
#[derive(Debug)]
pub struct LockGuard<'a, T: Send + Sync> {
	data: &'a mut T,
	lock: &'a RawLock,
}

#[derive(Debug)]
struct RawLock {
	is_hold: AtomicBool,
}

pub struct SpinLock<T: Send + Sync> {
	data: UnsafeCell<T>,
	lock: RawLock,
}

impl<T: Send + Sync> SpinLock<T> {
	fn new(data: T) -> Self {
		SpinLock { data: UnsafeCell::new(data), lock: RawLock { is_hold: AtomicBool::new(false) } }
	}
}

impl<'a, T: Send + Sync + Copy> SpinLock<T> {
	pub fn lock(&'a self) -> LockGuard<'a, T> {
		while self.lock.is_hold.compare_exchange(false, true, Ordering::Acquire, Ordering::Acquire).is_err() {
		}
		LockGuard::<'a, _> { 
			data: unsafe { &mut *self.data.get() as &mut T }, 
			lock: &self.lock 
		}
	}
}

impl<'a, T: Send + Sync> LockGuard<'a, T> {
	fn unlock(&self) {
		self.lock.is_hold.swap(false, Ordering::Release);
	}
}

impl<'a, T: Send + Sync> Deref for LockGuard<'a, T> {
	type Target = T;
	fn deref(&self) -> &Self::Target {
		&*self.data
	}
}

impl<'a, T: Send + Sync> DerefMut for LockGuard<'a, T> {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut *self.data
	}
}

#[test]
fn test() {
	let test: &'static SpinLock<_> = Box::leak(Box::new(SpinLock::new(0)));
	for _ in 0..10 {
		std::thread::spawn(move || {
			let mut guard = test.lock();
			*guard += 1;
			guard.unlock();
		});
	}
	println!("{}", test.lock().data);
}