//! A thread-safety cocurrent linked-list.
use std::{sync::Arc, marker::PhantomData};

use crate::Lock;
//   
pub struct LockCoupledList<'a, T, L>
where
    T: Send + Sync,
    L: Send + Sync + Lock<'a, Node<'a, L, T>>,
{
    head: Arc<Option<L>>,
    size: usize,
    _marker: PhantomData<&'a T>,
}

impl<'a, T, L> LockCoupledList<'a, T, L> 
where 
    T: Send + Sync,
    L: Send + Sync + Lock<'a, Node<'a, L, T>>,
{
    pub fn new() -> Self {
        LockCoupledList {
            head: Arc::new(None),
            size: 0,
            _marker: PhantomData::default(),
        }        
    }

    pub fn get(&self, i: usize) -> Option<Node<'a, L, T>> {
        if i >= self.size {
            return None; 
        };
        
        let curr = self.head.clone().unwrap().lock();
        for _ in 0..i {
            let next = *curr;
        };
    }
}

pub struct Node<'a, L, T> 
where
    T: Send + Sync,
    L: Send + Sync + Lock<'a, Node<'a, L, T>>,
{
    data: T,
    next: Arc<Option<L>>,
    _marker: PhantomData<&'a T>,
}

impl<'a, L, T> Node<'a, L, T>
where
    T: Send + Sync,
    L: Send + Sync + Lock<'a, Node<'a, L, T>>
{
    pub fn new(data: T) -> Self {
        Node {
            data,
            next: Arc::new(None),
            _marker: PhantomData::default(),
        }        
    } 
}

#[test]
fn test() {

}