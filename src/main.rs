use futures::{future, Future};
use hyper::{service::service_fn, Body, Method, Request, Response, Server, StatusCode};


const INDEX: &'static str = r#"
 <!doctype html>
 <html>
     <head>
         <title>Rust Microservice</title>
     </head>
     <body>
         <h3>Rust Microservice</h3>
     </body>
 </html>
 "#;

fn main() {
    let addr = ([0, 0, 0, 0], 8086).into();
    let builder = Server::bind(&addr);
    let server = builder.serve(|| service_fn(microservice_handler));
    let server = server.map_err(drop);
    hyper::rt::run(server);
}

fn microservice_handler(
    req: Request<Body>,
) -> impl Future<Item = Response<Body>, Error = hyper::Error> {
    match (req.method(), req.uri().path()) {
        (&Method::GET, "/") => future::ok(Response::new(INDEX.into())),
        _ => {
            let response = Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(Body::empty())
                .unwrap();
            future::ok(response)
        }
    }
}
