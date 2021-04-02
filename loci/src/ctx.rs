use crate::io::Duplex;
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

pub struct Context<'a> {
    pub boot: BootParam<'a>,
}

pub struct BootParam<'b> {
    pub cmd: &'b str,
    pub addr: &'b str,
}

impl<'b> BootParam<'b> {
    pub fn new(cmd: &'b str, addr: &'b str) -> Self {
        BootParam { cmd, addr }
    }
}

impl<'a> Context<'a> {
    pub fn new(boot: BootParam<'a>) -> Self {
        Context { boot }
    }
    pub async fn start_loci(&'static self, fd: RawFd, sender: Sender<()>) {
        info!("Starting loci...");
        let child_fd = dup_fd(fd);
        let std_listener = unsafe { std::net::TcpListener::from_raw_fd(fd) };
        let listener = TcpListener::from_std(std_listener).unwrap();
        for (source, _) in listener.accept().await {
            tokio::spawn(async move {
                let mut child = Command::new(self.boot.cmd)
                    .env("FD", child_fd.to_string())
                    .spawn()
                    .expect("Start child process error");
                let _ = child.wait().expect("Failed on wait child");
                sender.send(()).await;
                info!("Server has stopped");
            });
            //TODO: waiting for servant started with probe,with async sleep temporarily
            sleep(Duration::from_secs(1));
            let target = TcpStream::connect(self.boot.addr).await.unwrap();
            Duplex::new(source, target).start().await;
            close_fd(fd);
            break;
        }
    }

    pub async fn start_loci_holder(&'static self, fd: RawFd, s: Sender<()>, mut r: Receiver<()>) {
        loop {
            r.recv().await;
            let sc = s.clone();
            let standby = dup_fd(fd);
            tokio::spawn(async move {
                self.start_loci(standby, sc).await;
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
