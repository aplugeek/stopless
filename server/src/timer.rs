use std::os::unix::io::RawFd;
use std::sync::atomic::{AtomicI64, AtomicUsize, Ordering};
use std::sync::Arc;
use tokio::sync::mpsc::Sender;
use tokio::time::{sleep, Duration, Instant, Sleep};

pub struct ExitTimer {
    pub idle: usize,
    pub sender: Sender<()>,
    pub count: AtomicI64,
}

impl ExitTimer {
    pub fn new(sender: Sender<()>, idle: usize) -> Self {
        ExitTimer {
            sender,
            idle,
            count: AtomicI64::new(idle as i64),
        }
    }
    pub async fn start(&self) {
        loop {
            sleep(Duration::from_secs(1)).await;
            let count = self.count.fetch_sub(1, Ordering::SeqCst);
            if count == 0 {
                info!("Graceful shutdown server");
                self.sender.send(()).await;
                break;
            }
            info!("Exit timer count:{:?}", count - 1);
        }
    }
}
