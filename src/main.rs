use log::debug;
use std::sync::{Arc, Mutex};
use tokio::{
    io,
    select,
};

use crate::appguts::{AppGuts, UnsafeAppGuts};

mod cli;
mod control;
mod http;
mod tcp;
pub mod appguts;
pub mod mymiddleware;
pub mod ngrams;

#[tokio::main]
async fn main() -> io::Result<()> {
    let args = cli::get_cli_args();
    std::env::set_var("RUST_LOG", "info");
    env_logger::init();

    let guts: AppGuts = Arc::new(Mutex::new(UnsafeAppGuts::new()));
    
    let tcp_port_local = args.value_of("tcp_port_local").unwrap();
    let tcp_port_remote= args.value_of("tcp_port_clickhouse").unwrap();
    let http_port_local = args.value_of("http_port_local").unwrap();
    let http_port_remote= args.value_of("http_port_clickhouse").unwrap();
    let udp_control_port = args.value_of("udp_control_port").unwrap();
    let remote_ip = args.value_of("server").unwrap();
    
    let http_handler = http::start_http_handler(&http_port_local, &remote_ip, &http_port_remote, guts.clone());
    let tcp_handler = tcp::start_tcp_handler(&tcp_port_local, &remote_ip, &tcp_port_remote);
    let udp_handler = control::start_udp_handler(&udp_control_port, guts.clone());

    select!(
        Ok(()) = http_handler => {
            debug!("Http server shut down");
        } 
        Ok(()) = tcp_handler => {
            debug!("Tcp listener shut down")
        }
        Ok(()) = udp_handler => {
            debug!("App was stopped")
        }
    );

    Ok(())
}

