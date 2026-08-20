use flume::{Sender, Receiver, bounded};
use std::sync::Arc;

pub struct EventQueue<T> {
    sender: Sender<T>,
    receiver: Receiver<T>,
}

impl<T> EventQueue<T> {
    pub fn new(capacity: usize) -> Arc<Self> {
        let (sender, receiver) = bounded(capacity);
        Arc::new(Self { sender, receiver })
    }
    
    pub fn sender(&self) -> Sender<T> {
        self.sender.clone()
    }
    
    pub fn receiver(&self) -> Receiver<T> {
        self.receiver.clone()
    }
    
    pub fn len(&self) -> usize {
        self.sender.len()
    }
    
    pub fn is_empty(&self) -> bool {
        self.sender.is_empty()
    }
}