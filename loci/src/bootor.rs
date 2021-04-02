use crate::io::Duplex;
use async_trait::async_trait;
use std::os::unix::io::FromRawFd;
use std::os::unix::io::{AsRawFd, RawFd};
use std::process::Command;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;
use std::thread::sleep;
use tokio::io::DuplexStream;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::mpsc::{channel, Receiver, Sender};
use tokio::time::Duration;

pub struct Context {
    pub bootor: Arc<Bootor>,
}

pub struct Bootor {
    pub cmd: String,
    pub addr: String,
}

#[async_trait]
pub trait Boot {
    async fn start_loci(&self, fd: RawFd, sender: Sender<()>);
    async fn start_loci_holder(&self, fd: RawFd, s: Sender<()>, mut r: Receiver<()>);
}

impl Bootor {
    pub fn new(cmd: &str, addr: &str) -> Self {
        Bootor {
            cmd: cmd.into(),
            addr: addr.into(),
        }
    }
}

impl Context {
    pub fn new(boot: Bootor) -> Self {
        Context {
            bootor: Arc::new(boot),
        }
    }
}

#[async_trait]
impl Boot for Arc<Bootor> {
    async fn start_loci(&self, fd: RawFd, sender: Sender<()>) {
        info!("Starting loci...");
        let child_fd = dup_fd(fd);
        let std_listener = unsafe { std::net::TcpListener::from_raw_fd(fd) };
        let listener = TcpListener::from_std(std_listener).unwrap();
        let bp = self.clone();
        for (source, _) in listener.accept().await {
            tokio::spawn(async move {
                let mut child = Command::new(bp.cmd.as_str())
                    .env("FD", child_fd.to_string())
                    .spawn()
                    .expect("Start child process error");
                let _ = child.wait().expect("Failed on wait child");
                sender.send(()).await;
                info!("Server has stopped");
            });
            //TODO: waiting for servant started with probe,with async sleep temporarily
            sleep(Duration::from_secs(1));
            let target = TcpStream::connect(self.addr.as_str()).await.unwrap();
            close_fd(fd);
            Duplex::new(source, target).start().await;
            break;
        }
    }

    async fn start_loci_holder(&self, fd: RawFd, s: Sender<()>, mut r: Receiver<()>) {
        loop {
            r.recv().await;
            let sc = s.clone();
            let standby = dup_fd(fd);
            let bootor = self.clone();
            tokio::spawn(async move {
                bootor.start_loci(standby, sc).await;
            });
        }
    }
}

pub fn dup_fd(fd: RawFd) -> RawFd {
    let flag = nix::fcntl::F_DUPFD(fd);
    nix::fcntl::fcntl(fd, flag).unwrap()
}

pub fn close_fd(fd: RawFd) {
    unsafe {
        libc::close(fd);
    }
}
