use crate::ctx::{dup_fd, BootParam, Context};
use crate::io::Duplex;
use clap::{App, Arg, ArgMatches};
use futures::FutureExt;
use log::LevelFilter;
use std::os::unix::io::{AsRawFd, RawFd};
use tokio::net::TcpListener;
use tokio::sync::mpsc::channel;

#[macro_use]
extern crate log;
extern crate clap;

mod ctx;
mod io;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::builder().filter_level(LevelFilter::Info).init();
    let matches = App::new("Stopless for serverless")
        .version("1.0")
        .author("Jerry <Aplugeek@outlook.com>")
        .about("Does awesome things")
        .args_from_usage("-e, --endpoint=[server_bind] 'server bind address'")
        .args_from_usage("-c, --command=[exe] 'server boot command'")
        .get_matches();
    let ctx = boot_ctx(matches);
    let mut listener = TcpListener::bind(ctx.boot.addr).await.unwrap();
    let fd = listener.as_raw_fd();
    let standby = dup_fd(fd);
    let (tx, mut rx) = channel::<()>(1);
    let loci_tx = tx.clone();
    tokio::spawn(async move {
        ctx.start_loci(fd, loci_tx).await;
    });
    ctx.start_loci_holder(standby, tx, rx).await;
    Ok(())
}

fn boot_ctx(matches: ArgMatches) -> &'static Context {
    let addr = matches.value_of("endpoint").unwrap();
    let cmd = matches.value_of("command").unwrap();
    let ctx = Context::new(BootParam::new(cmd, addr));
    unsafe { std::mem::transmute(&ctx) }
}
