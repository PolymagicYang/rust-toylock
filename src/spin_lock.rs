use std::{sync::atomic::{ AtomicBool, Ordering }, ops::Deref};
// 自旋锁：持有锁的线程不断查看是否有其他线程在使用锁。

// 因为数据本身是可传递和同步的，所以锁也应该是可传递和可同步的。
unsafe impl<'s, T: Send + Sync> Send for LockGuard<'s, T> {}
unsafe impl<'s, T: Send + Sync> Sync for LockGuard<'s, T> {}

struct RawLock {
	is_hold: AtomicBool,
}

// 特别声明一个生命周期，确保锁保护的数据不会存活在锁的周期以外.
// T 必须可以被所有线程访问，或被发往别的线程.
struct LockGuard<'s, T: Send + Sync> {
	data: T,
	lock: RawLock,
}

impl<'s, T: Send + Sync> LockGuard<'s, T> {
	fn new(data: &'s T) -> Self {
		LockGuard { data: &data, lock: RawLock { is_hold: AtomicBool::new(false) } }
	}
}

impl<'s, T> Deref for LockGuard<'s, T> where T: Send + Sync {
	type Target = T;
	fn deref(&self) -> &Self::Target {
		&*self.data
	}
}

impl<'s, T: Send + Sync> LockGuard<'s, T> {
	pub fn lock(&self) {
		while self.lock.is_hold.compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed).is_err() {

		}
	}

	pub fn unlock(&self) {
		self.lock.is_hold.swap(false, Ordering::Release);
	}
}

#[test]
fn test() {
	let lock = LockGuard::new(0);
	println!("Hello!");
}