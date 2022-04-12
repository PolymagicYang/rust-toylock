pub mod spin_lock; 
pub mod ticket_lock;
pub mod CLH_lock;

/// Use RAII to protect data inside the box. 
trait Guard {
    fn unlock(&self);
}

trait Lock<'a, T: Send + Sync> {
    type L: Guard;
    fn lock(&self) -> Self::L;
}