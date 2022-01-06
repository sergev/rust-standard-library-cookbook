use hyper::service::{make_service_fn, service_fn};
use std::io;
use tokio;
use tokio_util::codec::{BytesCodec, FramedRead};

#[tokio::main]
async fn main() -> Result<(), hyper::Error> {
    let addr = "[::1]:3000".parse().expect("Failed to parse address");
    let file_service = make_service_fn(|_| async {
        Ok::<_, hyper::Error>(service_fn(run_file_server))
    });
    let server = hyper::Server::bind(&addr).serve(file_service);
    server.await
}

async fn run_file_server(req: hyper::Request<hyper::Body>) -> Result<hyper::Response<hyper::Body>, io::Error> {
    // Setting up our routes
    match (req.method(), req.uri().path()) {
        (&hyper::Method::GET, "/") => handle_root().await,
        (&hyper::Method::GET, path) => handle_get_file(path).await,
        _ => handle_invalid_method().await,
    }
}

// Because we don't want the entire server to block when serving a file,
// we are going to return a response wrapped in a future
type ResponseResult = Result<hyper::Response<hyper::Body>, io::Error>;
async fn handle_root() -> ResponseResult {
    // Send the landing page
    send_file_or_404("index.html").await
}

async fn handle_get_file(file: &str) -> ResponseResult {
    // Send whatever page was requested or fall back to a 404 page
    send_file_or_404(file).await
}

async fn handle_invalid_method() -> ResponseResult {
    // Send a page telling the user that the method he used is not supported
    send_file_or_404("invalid_method.html").await
        // Set the correct status code
        .or_else(|_| Ok(method_not_allowed()))
}

// Send a future containing a response with the requested file or a 404 page
async fn send_file_or_404(path: &str) -> ResponseResult {
    // Sanitize the input to prevent unwanted data access
    let path = sanitize_path(path);

    let response_future = try_to_send_file(&path)
        // try_to_send_file returns a future of Result<Response, io::Error>
        // turn it into a future of a future of Response with an error of hyper::Error
//TODO
//        .and_then(|response_result| response_result.map_err(|error| error.into()))
        // If something went wrong, send the 404 page instead
//        .or_else(|_| send_404())
        ;
    response_future.await
}

/// HTTP status code 404
fn not_found() -> hyper::Response<hyper::Body> {
    const NOTFOUND: &[u8] = b"Not Found";
    hyper::Response::builder()
        .status(hyper::StatusCode::NOT_FOUND)
        .body(NOTFOUND.into())
        .unwrap()
}

fn method_not_allowed() -> hyper::Response<hyper::Body> {
    const NOTALLOWED: &[u8] = b"Method not allowed";
    hyper::Response::builder()
        .status(hyper::StatusCode::METHOD_NOT_ALLOWED)
        .body(NOTALLOWED.into())
        .unwrap()
}

// Return a requested file in a future of Result<Response, io::Error>
// to indicate whether it exists or not
async fn try_to_send_file(filename: &str) -> ResponseResult {
    // Prepend "files/" to the file
    let path = path_on_disk(filename);

    if let Ok(file) = tokio::fs::File::open(path).await {
        println!("Sending file: {}", filename);
        let stream = FramedRead::new(file, BytesCodec::new());
        let body = hyper::Body::wrap_stream(stream);
        // Detect the content type by checking the file extension
        // or fall back to plaintext
        let content_type = get_content_type(&filename).unwrap_or("text/plain; charset=utf-8".to_string());
        let response = hyper::Response::builder()
            .header(hyper::header::CONTENT_TYPE, content_type)
            .body(body)
            .unwrap();
        return Ok(response);
    }

    println!("Failed to find file: {}", filename);
    Ok(not_found())
}

async fn send_404() -> ResponseResult {
    // Try to send our 404 page
    let response_future = try_to_send_file("not_found.html")
//TODO
//        .and_then(|response_result| {
//            Ok(response_result.unwrap_or(
//                // If the 404 page doesn't exist, sent fallback text instead
//                const ERROR_MSG: &str = "Failed to find \"File not found\" page. How ironic\n";
//                hyper::Response::builder()
//                    .status(hyper::StatusCode::NOT_FOUND)
//                    .body(ERROR_MSG.into())
//                    .unwrap()
//            ))
//        })
        ;
    response_future.await
}

fn sanitize_path(path: &str) -> String {
    // Normalize the separators for the next steps
    path.replace("\\", "/")
        // Prevent the user from going up the filesystem
        .replace("../", "")
        // If the path comes straigh from the router,
        // it will begin with a slash
        .trim_start_matches(|c| c == '/')
        // Remove slashes at the end as we only serve files
        .trim_end_matches(|c| c == '/')
        .to_string()
}

fn path_on_disk(path_to_file: &str) -> String {
    "files/".to_string() + path_to_file
}

fn get_content_type(file: &str) -> Option<String> {
    // Check the file extension and return the respective MIME type
    let pos = file.rfind('.')? + 1;
    let mime_type = match &file[pos..] {
        "txt" => "text/plain; charset=utf-8",
        "html" => "text/html; charset=utf-8",
        "css" => "text/css",
        // This list can be extended for all types your server should support
        _ => return None,
    };
    Some(String::from(mime_type))
}
