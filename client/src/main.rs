use tokio::{io::AsyncWriteExt, net::TcpStream, time::{sleep, Duration}};

#[tokio::main]
async fn main() {
    let n = 100;
    let handles: Vec<_> = (0..n).map(|i| tokio::spawn(async move {
        let mut stream = TcpStream::connect("127.0.0.1:8080").await.unwrap();

        stream.write_all(b"ping\n").await.unwrap();

        sleep(Duration::from_secs(5)).await;

        if i % 10 == 0 { return; }

        stream.shutdown().await.ok();
    })).collect();
    for h in handles { h.await.ok(); }
}