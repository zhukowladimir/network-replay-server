use log::{error, info, debug};
use tokio::io::{self, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

pub async fn start_tcp_handler(local_port: &str, remote_ip: &str, remote_port: &str) -> io::Result<()> {
    debug!("start_tcp_proxy");

    let local_addr = format!("0.0.0.0:{}", local_port);
    let remote_addr = format!("{}:{}", remote_ip, remote_port);
    debug!("local addr: {:?}", local_addr);
    debug!("remote addr: {:?}", remote_addr);

    let listener = TcpListener::bind(&local_addr).await?;
    info!("Listening TCP on {:?}", local_addr);

    loop {
        let (socket, client_addr) = listener.accept().await?;
        let remote_addr = remote_addr.to_owned();
        info!("Client {} accepted", &client_addr);

        tokio::spawn(async move {
            match proxy_to_remote(socket, remote_addr.as_str()).await {
                Ok(_) => info!("Client {} disconnected", &client_addr),
                Err(e) => error!("{}", e),
            }
        });
    }
}

async fn proxy_to_remote(mut origin: TcpStream, remote: &str) -> io::Result<()> {
    debug!("proxy_to_remote");

    let mut remote = TcpStream::connect(remote).await?;

    let (mut rc, mut wc) = origin.split();
    let (mut rr, mut wr) = remote.split();

    let local_to_remote = async {
        io::copy(&mut rc, &mut wr).await?;
        // proxy_reader_writer("Client->Remote:", &mut rc, &mut wr).await?;
        wr.shutdown().await
    };

    let remote_to_local = async {
        io::copy(&mut rr, &mut wc).await?;
        // proxy_reader_writer("Remote->Client:", &mut rr, &mut wc).await?;
        wc.shutdown().await
    };

    tokio::try_join!(local_to_remote, remote_to_local)?;

    Ok(())
}

// async fn proxy_reader_writer<'a, R: ?Sized, W: ?Sized>(
//     direction: &str,
//     reader: &'a mut R,
//     writer: &'a mut W,
// ) -> std::io::Result<()>
// where
//     R: AsyncRead + Unpin,
//     W: AsyncWrite + Unpin,
// {
//     debug!("proxy_reader_writer");

//     let bytes_submitted = io::copy(reader, writer).await?;
//     info!("{} {}", direction, bytes_submitted);

//     Ok(())
// }