use tokio::time::{sleep, Duration};
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

pub struct RateLimiter {
    last_request: AtomicU64,
    min_interval_ms: u64,
}

impl RateLimiter {
    pub fn new(_max_per_second: u32, min_interval_ms: u64) -> Arc<Self> {
        Arc::new(Self {
            last_request: AtomicU64::new(0),
            min_interval_ms,
        })
    }
    
    pub async fn acquire(&self) {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;
        
        let last = self.last_request.load(Ordering::Relaxed);
        
        if last > 0 && now > last {
            let elapsed = now - last;
            
            if elapsed < self.min_interval_ms {
                sleep(Duration::from_millis(self.min_interval_ms - elapsed)).await;
            }
        }
        
        let now_after = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;
        
        self.last_request.store(now_after, Ordering::Relaxed);
    }
}