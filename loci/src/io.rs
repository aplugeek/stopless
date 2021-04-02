use futures::join;
use futures::task::Context;
use futures::{FutureExt, TryFutureExt};
use tokio::io::{copy, split, AsyncRead, AsyncWrite};
use tokio::macros::support::{Future, Pin, Poll};
use tokio::net::TcpStream;

pub struct Duplex<I, O>
where
    I: AsyncRead + AsyncWrite + Unpin,
    O: AsyncRead + AsyncWrite + Unpin,
{
    half_in: I,
    half_out: O,
}

// impl<C,F> Future for Duplex<C,F> {
//     type Output = T;
//
//     fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
//         Pin::new(&mut self.inner).poll(cx)
//     }
// }

impl<I, O> Duplex<I, O>
where
    I: AsyncRead + AsyncWrite + Unpin,
    O: AsyncRead + AsyncWrite + Unpin,
{
    pub fn new(input: I, output: O) -> Self {
        Duplex {
            half_in: input,
            half_out: output,
        }
    }

    pub async fn start(&mut self) {
        let (mut sr, mut sw) = split(&mut self.half_in);
        let (mut tr, mut tw) = split(&mut self.half_out);
        let sink = copy(&mut sr, &mut tw);
        let stream = copy(&mut tr, &mut sw);
        join!(sink, stream);
    }
}
