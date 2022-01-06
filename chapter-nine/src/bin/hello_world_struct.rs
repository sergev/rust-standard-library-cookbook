use futures::future::Future;
use std::pin::Pin;
use std::net::SocketAddr;
use std::task::{Context, Poll};
use tokio;

const MESSAGE: &str = "Hello World!";

#[tokio::main]
async fn main() -> Result<(), hyper::Error> {
    // [::1] is the loopback address for IPv6, 3000 is a port
    let addr = "[::1]:3000".parse().expect("Failed to parse address");
    run_with_service_struct(&addr).await
}

// The following function does the same, but uses an explicitely created
// struct HelloWorld that implements the Service trait
async fn run_with_service_struct(addr: &SocketAddr) -> Result<(), hyper::Error> {
    let server = hyper::Server::bind(addr).serve(MakeHelloWorld {});

    // Run forever-ish...
    println!("Listening on http://{}", addr);
    server.await
}

struct MakeHelloWorld;
impl<T> hyper::service::Service<T> for MakeHelloWorld {
    type Response = HelloWorld;
    type Error = hyper::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, _: &mut Context) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, _: T) -> Self::Future {
        println!("Got a connection!");
        let fut = async move { Ok(HelloWorld {}) };
        Box::pin(fut)
    }
}

struct HelloWorld;
impl hyper::service::Service<hyper::Request<hyper::Body>> for HelloWorld {
    // Implementing a server requires specifying all involved types
    type Response = hyper::Response<hyper::Body>;
    type Error = hyper::Error;
    // The future that wraps your eventual Response
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, _: &mut Context) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, _: hyper::Request<hyper::Body>) -> Self::Future {
        println!("Sent reply.");
        let response = hyper::Response::builder()
            // Add header specifying content type as plain text
            .header(hyper::header::CONTENT_TYPE, "text/plain; charset=utf-8")
            // Add body with our message
            .body(hyper::Body::from(MESSAGE))
            .unwrap();
        // In contrast to service_fn, we need to explicitely return a future
        Box::pin(futures::future::ok(response))
    }
}
