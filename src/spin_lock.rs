use std::{sync::{atomic::{ AtomicBool, Ordering }, Arc}, ops::{Deref, DerefMut}};
// 自旋锁：持有锁的线程不断查看是否有其他线程在使用锁。

// 因为数据本身是可传递和同步的，所以锁也应该是可传递和可同步的。
unsafe impl<T: Send + Sync> Send for SpinLock<T> {}
unsafe impl<T: Send + Sync> Sync for SpinLock<T> {}

struct RawLock {
	is_hold: AtomicBool,
}

// 特别声明一个生命周期，确保锁保护的数据不会存活在锁的周期以外.
// T 必须可以被所有线程访问，或被发往别的线程.
pub struct LockGuard<'a, T: Send + Sync> {
	data: &'a mut T,
	lock: &'a RawLock,
}

pub struct SpinLock<T: Send + Sync> {
	data: T,
	lock: RawLock,
}

impl<T: Send + Sync> SpinLock<T> {
	fn new(data: T) -> Self {
		SpinLock { data, lock: RawLock { is_hold: AtomicBool::new(false) } }
	}
}

impl<T: Send + Sync> SpinLock<T> {
	pub fn lock<'a>(self) -> LockGuard<'a, T> {
		while self.lock.is_hold.compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed).is_err() {

		}
		LockGuard { data: &mut self.data, lock: &self.lock }
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
	let mut test = Arc::new(SpinLock::new(0));
	for _ in 0..10 {
		let test = test.clone();
		std::thread::spawn(move || {
			let guard = test.lock();
			*guard += 1;
		});
	}
}