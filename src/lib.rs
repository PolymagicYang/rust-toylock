use std::ops::Deref;

// Covered implmentation:
// impl<P0..Pn> ForeignTrait<T1..Tn> for T0

pub mod spin_lock; 
pub mod ticket_lock;
pub mod CLH_lock;
pub mod CMS_lock;
pub mod list;

/// Use RAII to protect data inside the box. 
pub trait Guard: Deref
where 
    Self::Target: Sized + Send + Sync
{
    fn unlock(&self);
}

pub trait Lock<'a, T> 
where 
    T: Send + Sync + Sized
{
    type G: Guard;
    
    // fn lock(&self) -> Self::L;
    fn lock(&'a self) -> Self::G;
}

#[test]
fn test_t() {
    let t = Damn {};
    let t1 = &t;
    foo(t1);
    t1.test();
}

trait Test {
    fn test(&self) {

    }
}

fn foo<T: Test>(t: T) {
    t.test();
}

impl<T> Test for &T {
    fn test(&self) {
        
    } 
}

struct Damn {

}

impl Test for Damn {
    fn test(&self) {
        
    }    
}
