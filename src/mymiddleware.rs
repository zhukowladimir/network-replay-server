use actix_web::{
    body::{self, EitherBody, MessageBody},
    dev::{self, Service, ServiceRequest, ServiceResponse, Transform, Payload},
    web::{Bytes, BytesMut},
    error::PayloadError,
    Error,
    HttpMessage,
    HttpResponseBuilder
};
use std::{
    future::{self, ready, Ready},
    pin::Pin,
    rc::Rc,
};
use futures::Stream;
use futures_util::{future::LocalBoxFuture, stream::{self, StreamExt}};
use log::{debug};

pub struct Logging;

impl<S: 'static, B> Transform<S, ServiceRequest> for Logging
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: MessageBody + 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type InitError = ();
    type Transform = LoggingMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(LoggingMiddleware {
            service: Rc::new(service),
        }))
    }
}

pub struct LoggingMiddleware<S> {
    // This is special: We need this to avoid lifetime issues.
    service: Rc<S>,
}

impl<S, B> Service<ServiceRequest> for LoggingMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: MessageBody + 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    dev::forward_ready!(service);

    fn call(&self, mut req: ServiceRequest) -> Self::Future {
        let svc = self.service.clone();

        Box::pin(async move {
            let mut body = BytesMut::new();
            let mut stream = req.take_payload();
            while let Some(chunk) = stream.next().await {
                body.extend_from_slice(&chunk?);
            }
            let body = body.freeze();

            debug!("HTTP: {peer_addr} --> {method} {path} {version:?}\n{headers}BODY:{body:?}\n",
                    peer_addr=req.connection_info().realip_remote_addr().unwrap_or(""),
                    method=req.method(),
                    path=req.path(),
                    version=req.version(),
                    headers=req.headers().iter()
                                        .map(|(key, value)| format!("{}: {}\n", key.as_str(), std::str::from_utf8(value.as_bytes()).unwrap_or("Invalid UTF-8 value")))
                                        .collect::<String>(),
                    body=&body
                    );

            let payload = body.slice(..);
            let single_part: Result<Bytes, PayloadError> = Ok(payload);
            let in_memory_stream = stream::once(future::ready(single_part));
            let pinned_stream: Pin<Box<dyn Stream<Item=Result<Bytes, PayloadError>>>> = Box::pin(in_memory_stream);
            let in_memory_payload: Payload = pinned_stream.into();
            req.set_payload(in_memory_payload);


            let resp = svc.call(req).await?.map_into_boxed_body();

            let req_clone = resp.request().clone();
            let resp_status = resp.status().clone();
            let resp_error = resp.response().error().map(|error| format!(" Origin Error: {}", error)).unwrap_or("".to_string());
            let resp_headers = resp.headers().clone();
            let body = body::to_bytes(resp.into_body()).await.unwrap_or(Bytes::new());
            
            debug!("HTTP: <-- {status}{error}\n{headers}BODY:{body:?}\n",
                  status=resp_status,
                  error=resp_error,
                  headers=resp_headers.iter()
                                      .map(|(key, value)| format!("{}: {}\n", key.as_str(), std::str::from_utf8(value.as_bytes()).unwrap_or("Invalid UTF-8 value")))
                                      .collect::<String>(),
                  body=&body
                );
            
            let mut resp_clone = HttpResponseBuilder::new(resp_status);
            for (header_name, header_value) in resp_headers {
                resp_clone.insert_header((header_name.as_str(), header_value));
            }
            let resp_clone = resp_clone.body(body.to_vec());


            Ok(ServiceResponse::new(
                req_clone,
                resp_clone.map_into_right_body(),
            ))
        })
    }
}
