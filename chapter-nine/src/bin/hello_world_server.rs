use std::net::SocketAddr;
use tokio;
use hyper::service::{make_service_fn, service_fn};

const MESSAGE: &str = "Hello World!";

#[tokio::main]
async fn main() -> Result<(), hyper::Error> {
    // [::1] is the loopback address for IPv6, 3000 is a port
    let addr = "[::1]:3000".parse().expect("Failed to parse address");
    run_with_service_function(&addr).await
}

async fn run_with_service_function(addr: &SocketAddr) -> Result<(), hyper::Error> {
    // Hyper is based on Services, which are construct that
    // handle how to respond to requests.
    // const_service and service_fn are convenience functions
    // that build a service out of a closure
    let hello_world = make_service_fn(move |_| async {
        println!("Got a connection!");
        // Return a Response with a body of type hyper::Body
        Ok::<_, hyper::Error>(service_fn(move |_req| async {
            println!("Sent reply.");
            let response = hyper::Response::builder()
                // Add header specifying content type as plain text
                .header(hyper::header::CONTENT_TYPE, "text/plain; charset=utf-8")
                // Add body with our message
                .body(hyper::Body::from(MESSAGE))
                .unwrap();
            Ok::<_, hyper::Error>(response)
        }))
    });

    let server = hyper::Server::bind(addr).serve(hello_world);

    // Run forever-ish...
    println!("Listening on http://{}", addr);
    server.await
}
