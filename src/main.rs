use hyper::{Server, service::service_fn_ok, Response, Body, rt::Future};

fn main() {
    let addr = ([0,0,0,0],8086).into();
    let builder = Server::bind(&addr);
    let server = 
        builder.serve(|| 
            service_fn_ok(|_| {
            Response::new(Body::from("Hello, world!"))
        }));
    let server = server.map_err(drop);
    hyper::rt::run(server);
}
