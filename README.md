# network-replay-server 
> a.k.a. mock-server for ClickHouse

## Setup

Install Rust and Cargo
```
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

## Usage
### App

```
cargo run -- --first_argument_name first_value --second_argument_name second_value
```
```
# for help
cargo run -- --help
```

### UDP client for commands
```
nc -u 0.0.0.0 8766
nc -u network-replay-server_ip udp_control_port
```
command list: 
 - `stop`
 - `show db`
 - `change state`

### Http requests
[https://clickhouse.com/docs/en/interfaces/http](https://clickhouse.com/docs/en/interfaces/http)

### Native client
```
./clickhouse client -h 0.0.0.0 --port 1313
./clickhouse client -h network-replay-server_ip --port tcp_port
```
[https://clickhouse.com/docs/en/interfaces/cli/](https://clickhouse.com/docs/en/interfaces/cli/)
