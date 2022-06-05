use log::{self, info, debug, error};
use tokio::{
    io,
    net::UdpSocket,
    select,
    sync::mpsc::{self, Sender},
};

use crate::appguts::AppGuts;

pub async fn start_udp_handler(port: &str, guts: AppGuts) -> io::Result<()> {
    let (commands_sender, mut commands_receiver) = mpsc::channel::<String>(16);

    let addr = format!("localhost:{}", &port);
    let control = UdpSocket::bind(&addr).await?;
    info!("Listening UDP commands on {:?}", addr);

    loop {
        let guts = guts.clone();

        select!{
            Ok(()) = act(&control, commands_sender.clone()) => {
            }
            Some(command) = commands_receiver.recv() => {
                if command == "stop" {
                    break;
                } else if command == "change state" {
                    let mut guts = guts.lock().unwrap();
                    guts.change_state();
                } else if command == "show db" {
                    let guts = guts.lock().unwrap();
                    guts.show_data();
                } else {
                    error!("invalid command: {}", command)
                }
            }
        }
    }

    Ok(())
}

async fn act(control: &UdpSocket, commands_sender: Sender<String>) -> io::Result<()> {
    let mut buf = vec![0u8; 256];
    let (len, admin_address) = control.recv_from(&mut buf).await?;
    buf.resize(len, 0);

    let command = String::from_utf8(buf).unwrap();
    debug!("Got command {:?} from {:?}", &command, admin_address);

    control.send_to(b"Ack\n", admin_address).await?;

    commands_sender.send(command.trim().into()).await.unwrap();

    Ok(())
}
