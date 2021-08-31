use crate::bootor::{dup_fd, Boot, Bootor, Context};
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

mod bootor;
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
    let c = boot_ctx(matches);
    let ctx: &'static Context = unsafe { std::mem::transmute(&c) };
    info!("{:?}", ctx.bootor.addr);
    let mut listener = TcpListener::bind(ctx.bootor.addr.as_str()).await.unwrap();
    let fd = listener.as_raw_fd();
    let (tx, mut rx) = channel::<()>(1);
    let loci_tx = tx.clone();
    tokio::spawn(async move {
        ctx.bootor.start_loci(fd, loci_tx).await;
    });
    ctx.bootor.start_loci_holder(fd, tx, rx).await;
    Ok(())
}

fn boot_ctx(matches: ArgMatches) -> Context {
    let addr = matches.value_of("endpoint").unwrap();
    let cmd = matches.value_of("command").unwrap();
    Context::new(Bootor::new(cmd, addr))
}
