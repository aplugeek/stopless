use crate::timer::ExitTimer;
use futures::{join, select};
use hyper::server::conn::AddrIncoming;
use hyper::server::Builder;
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Request, Response, Server, StatusCode};
use log::LevelFilter;
use std::net::TcpListener as StdTcpListener;
use std::os::unix::io::FromRawFd;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::mpsc::channel;

mod timer;

#[macro_use]
extern crate log;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::builder().filter_level(LevelFilter::Info).init();
    let (tx, mut rx) = channel::<()>(1);
    let timer = ExitTimer::new(tx, 10);
    let exit_timer: &'static ExitTimer = unsafe { std::mem::transmute(&timer) };
    let incoming = from_fd()?;
    let server = incoming
        .serve(make_service_fn(move |_| async move {
            Ok::<_, hyper::Error>(service_fn(move |req: Request<Body>| async move {
                exit_timer.count.swap(10, Ordering::SeqCst);
                info!("Connection incoming,reset exit timer");
                serve().await
            }))
        }))
        .with_graceful_shutdown(async {
            rx.recv().await;
            info!("Server has graceful shutdown!")
        });
    let timer = exit_timer.start();
    join!(timer, server);
    Ok(())
}

async fn serve() -> std::io::Result<Response<Body>> {
    let mut resp = Response::new(Body::empty());
    *resp.status_mut() = StatusCode::OK;
    Ok(resp)
}

fn from_fd() -> Result<Builder<AddrIncoming>, Box<dyn std::error::Error>> {
    let fd = std::env::var("FD").unwrap().parse::<i32>().unwrap();
    let std_listener = unsafe { StdTcpListener::from_raw_fd(fd) };
    let incoming = Server::from_tcp(std_listener)?;
    Ok(incoming)
}
