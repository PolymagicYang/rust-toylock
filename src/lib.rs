use spin_lock::LockGuard;

pub mod spin_lock;
pub mod ticket_lock;

// todo: abstract the Lock and Lockguard trait.