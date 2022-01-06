use hyper::{Method, StatusCode};
use hyper::service::{make_service_fn, service_fn};
use std::net::SocketAddr;
use tokio;

static NOTFOUND: &[u8] = b"Not Found";

#[tokio::main]
async fn main() -> Result<(), hyper::Error> {
    let addr = "[::1]:3000".parse().expect("Failed to parse address");
    run_echo_server(&addr).await
}

async fn run_echo_server(addr: &SocketAddr) -> Result<(), hyper::Error> {
    let echo_service = make_service_fn(move |_| async {
        Ok::<_, hyper::Error>(service_fn(move |req: hyper::Request<hyper::Body>| async {
        // An easy way to implement routing is
        // to simply match the request's path
        match (req.method(), req.uri().path()) {
            (&Method::GET, "/") => handle_root(),
            (&Method::POST, "/echo") => handle_echo(req),
            _ => handle_not_found(),
        }
    }))});

    let server = hyper::Server::bind(addr).serve(echo_service);
    server.await
}

type ResponseResult = Result<hyper::Response<hyper::Body>, hyper::Error>;
fn handle_root() -> ResponseResult {
    const MSG: &str = "Try doing a POST at /echo";
    let response = hyper::Response::builder()
        .header(hyper::header::CONTENT_TYPE, "text/plain; charset=utf-8")
        .body(hyper::Body::from(MSG))
        .unwrap();
    Ok(response)
}

fn handle_echo(req: hyper::Request<hyper::Body>) -> ResponseResult {
    // The echoing is implemented by setting the response's
    // body to the request's body
    let response = hyper::Response::builder()
        .header(hyper::header::CONTENT_TYPE, "text/plain; charset=utf-8")
        .body(req.into_body())
        .unwrap();
    Ok(response)
}

fn handle_not_found() -> ResponseResult {
    // Return a 404 for every unsupported route
    let response = hyper::Response::builder()
        .status(StatusCode::NOT_FOUND)
        .body(NOTFOUND.into())
        .unwrap();
    Ok(response)
}
