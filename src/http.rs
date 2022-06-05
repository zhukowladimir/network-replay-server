use actix_web::{
    error::{self, PayloadError}, 
    web, App, Error, HttpRequest, HttpResponse, HttpServer,
};
use awc::Client;
use futures::Stream;
use futures_util::stream::{self, StreamExt};
use log::{info, debug};
use num_cpus;
use std::{cmp, future, pin::Pin};
use tokio::io;
use url::Url;

use crate::{mymiddleware::Logging, appguts::AppGuts, ngrams::{MiddlewareDataHttp}};

pub async fn start_http_handler(local_port: &str, remote_ip: &str, remote_port: &str, guts: AppGuts) -> io::Result<()> {
    let forward_url = format!("http://{}:{}", &remote_ip, &remote_port);
    let forward_url = Url::parse(&forward_url).unwrap();

    info!(
        "Starting HTTP server at http://{}:{}",
        "0.0.0.0",
        &local_port,
    );

    info!("Forwarding to {forward_url}");

    let cpu_num = cmp::max(num_cpus::get() / 2, 1);

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(Client::default()))
            .app_data(web::Data::new(forward_url.clone()))
            .app_data(web::Data::new(guts.clone()))
            // .wrap(middleware::Logger::default())
            .wrap(Logging)
            .default_service(web::to(forward))
    })
    .bind(("0.0.0.0", local_port.parse::<u16>().unwrap()))?
    .workers(cpu_num)
    .run()
    .await
}

async fn forward(
    req: HttpRequest,
    mut payload: web::Payload,
    guts: web::Data<AppGuts>,
    url: web::Data<Url>,
    client: web::Data<Client>,
) -> Result<HttpResponse, Error> {
    let mut new_url = url.get_ref().clone();
    new_url.set_path(req.uri().path());
    new_url.set_query(req.uri().query());

    let forwarded_req = client
        .request_from(new_url.as_str(), req.head())
        .no_decompress();
    let forwarded_req = match req.head().peer_addr {
        Some(addr) => forwarded_req.insert_header(("x-forwarded-for", format!("{}", addr.ip()))),
        None => forwarded_req,
    };

    let mut req_body = web::BytesMut::new();
    while let Some(chunk) = payload.next().await {
        req_body.extend_from_slice(&chunk?);
    }
    let req_body = req_body.freeze();

    let mut req_body_clone = web::BytesMut::with_capacity(req_body.len());
    req_body_clone.resize(req_body.len(), 0);
    req_body_clone.clone_from_slice(&req_body[..]);

    let req_body_str = std::str::from_utf8(&req_body_clone[..]).unwrap_or("Invalid UTF-8 value").to_string();
    debug!("body_str: {:?}", &req_body_str);

    let payload = req_body.slice(..);
    let single_part: Result<web::Bytes, PayloadError> = Ok(payload);
    let in_memory_stream = stream::once(future::ready(single_part));
    let payload: Pin<Box<dyn Stream<Item=Result<web::Bytes, PayloadError>>>> = Box::pin(in_memory_stream);

    ///////////////////////////

    let is_record: bool;
    {
        let guts = guts.lock().unwrap();
        is_record = guts.is_record_state();
    }

    ///////////////////////////

    if is_record {

        let mut resp = forwarded_req
            .send_stream(payload)
            .await
            .map_err(error::ErrorInternalServerError)?;

        let resp_status = resp.status().clone();
        let resp_headers = resp.headers().clone();

        let mut resp_body = web::BytesMut::new();
        while let Some(chunk) = resp.next().await {
            resp_body.extend_from_slice(&chunk?);
        }
        let resp_body = resp_body.freeze();

        let mut resp_body_clone = web::BytesMut::with_capacity(resp_body.len());
        resp_body_clone.resize(resp_body.len(), 0);
        resp_body_clone.clone_from_slice(&resp_body[..]);
        debug!("resp_body_clone: {:?}", &resp_body_clone);

        let resp = resp_body.slice(..);
        let single_part: Result<web::Bytes, PayloadError> = Ok(resp);
        let in_memory_stream = stream::once(future::ready(single_part));
        let resp: Pin<Box<dyn Stream<Item=Result<web::Bytes, PayloadError>>>> = Box::pin(in_memory_stream);

        let mut client_resp = HttpResponse::build(resp_status);
        for (header_name, header_value) in resp_headers.iter().filter(|(h, _)| *h != "connection") {
            client_resp.insert_header((header_name.clone(), header_value.clone()));
        }
        let client_resp = client_resp.streaming(resp);

        {
            let mut guts = guts.lock().unwrap();
            guts.insert_data(req_body_str, resp_body_clone.into(), Some(MiddlewareDataHttp::new(resp_status, resp_headers)));
        }

        return Ok(client_resp);

    } else {

        let guts = guts.lock().unwrap();

        let (resp, status_headers) = guts.find_best_answer(req_body_str);
        let (resp_status, resp_headers) = status_headers.split();

        let single_part: Result<web::Bytes, PayloadError> = Ok(resp);
        let in_memory_stream = stream::once(future::ready(single_part));
        let resp: Pin<Box<dyn Stream<Item=Result<web::Bytes, PayloadError>>>> = Box::pin(in_memory_stream);

        let mut client_resp = HttpResponse::build(resp_status);
        for (header_name, header_value) in resp_headers.iter().filter(|(h, _)| *h != "connection") {
            client_resp.insert_header((header_name.clone(), header_value.clone()));
        }
        let client_resp = client_resp.streaming(resp);

        return Ok(client_resp);
    }

}
