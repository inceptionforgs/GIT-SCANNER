use tokio::sync::Semaphore;
use tokio::time::{sleep, Duration, Instant};
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

pub struct RateLimiter {
    semaphore: Arc<Semaphore>,
    last_request: AtomicU64,
    min_interval_ms: u64,
    max_per_second: u32,
}

impl RateLimiter {
    pub fn new(max_per_second: u32, min_interval_ms: u64) -> Arc<Self> {
        Arc::new(Self {
            semaphore: Arc::new(Semaphore::new(max_per_second as usize)),
            last_request: AtomicU64::new(0),
            min_interval_ms,
            max_per_second,
        })
    }
    
    pub async fn acquire(&self) {
        // Semaphore acquire for rate limiting
        let _permit = self.semaphore.clone().acquire_owned().await.unwrap();
        
        // Minimum interval between requests
        let now = Instant::now();
        let last = self.last_request.load(Ordering::Relaxed);
        let elapsed = now.elapsed().as_millis() as u64;
        
        if elapsed < self.min_interval_ms {
            sleep(Duration::from_millis(self.min_interval_ms - elapsed)).await;
        }
        
        self.last_request.store(now.elapsed().as_millis() as u64, Ordering::Relaxed);
        
        // Release semaphore after interval
        tokio::spawn(async move {
            sleep(Duration::from_millis(1000 / self.max_per_second as u64)).await;
        });
    }
    
    pub fn max_per_second(&self) -> u32 {
        self.max_per_second
    }
}