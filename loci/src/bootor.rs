use crate::io::Duplex;
use async_trait::async_trait;
use std::convert::TryFrom;
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
        let bp = self.clone();
        select_fd(fd);
        close_fd(fd);
        tokio::spawn(async move {
            let mut child = Command::new(bp.cmd.as_str())
                .env("FD", child_fd.to_string())
                .spawn()
                .expect("Start child process error");
            let _ = child.wait().expect("Failed on wait child");
            sender.send(()).await;
            info!("Server has stopped");
        });
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

pub fn select_fd(fd: RawFd) {
    let mut fd_set = nix::sys::select::FdSet::new();
    fd_set.insert(fd);
    nix::sys::select::select(None, Some(&mut fd_set), None, None, None);
}
