use clap::{Command, Arg, ArgMatches};

pub fn get_cli_args() -> ArgMatches {
    Command::new("proxy")
        .version("0.1")
        .author("zhukowladimir <vazhukov_1@edu.hse.ru>")
        .about("TCP Proxy")
        .arg(
            Arg::new("server")
                .short('s')
                .long("server")
                .value_name("ADDRESS")
                .default_value("localhost")
                .help("The ip address of the ClickHouse server, 127.0.0.1 for example")
                .takes_value(true)
                .required(false),
        )
        .arg(
            Arg::new("http_port_local")
                .long("http_port_local")
                .value_name("PORT")
                .default_value("8123")
                .help("Port for unencrypted HTTP queries on local host")
                .takes_value(true)
                .required(false),
        )
        .arg(
            Arg::new("https_port_local")
                .long("https_port_local")
                .value_name("PORT")
                .default_value("8443")
                .help("Port for encrypted HTTPS queries on local host")
                .takes_value(true)
                .required(false),
        )
        .arg(
            Arg::new("tcp_port_local")
                .long("tcp_port_local")
                .value_name("PORT")
                .default_value("9000")
                .help("Port for unencrypted native TCP/IP queries on local host")
                .takes_value(true)
                .required(false),
        )
        .arg(
            Arg::new("tcp_port_secure_local")
                .long("tcp_port_secure_local")
                .value_name("PORT")
                .default_value("9440")
                .help("Port for TLS-encrypted native TCP/IP queries on local host")
                .takes_value(true)
                .required(false),
        )
        .arg(
            Arg::new("http_port_clickhouse")
                .long("http_port_clickhouse")
                .value_name("PORT")
                .default_value("8123")
                .help("Port for unencrypted HTTP queries on server host")
                .takes_value(true)
                .required(false),
        )
        .arg(
            Arg::new("https_port_clickhouse")
                .long("https_port_clickhouse")
                .value_name("PORT")
                .default_value("8443")
                .help("Port for encrypted HTTPS queries on server host")
                .takes_value(true)
                .required(false),
        )
        .arg(
            Arg::new("tcp_port_clickhouse")
                .long("tcp_port_clickhouse")
                .value_name("PORT")
                .default_value("9000")
                .help("Port for unencrypted native TCP/IP queries on server host")
                .takes_value(true)
                .required(false),
        )
        .arg(
            Arg::new("tcp_port_secur_clickhousee")
                .long("tcp_port_secure_clickhouse")
                .value_name("PORT")
                .default_value("9440")
                .help("Port for TLS-encrypted native TCP/IP queries on server host")
                .takes_value(true)
                .required(false),
        )
        .arg(
            Arg::new("udp_control_port")
                .long("udp_control_port")
                .value_name("PORT")
                .default_value("8766")
                .help("Port for UDP control commands")
                .takes_value(true)
                .required(false),
        )
        .get_matches()
}