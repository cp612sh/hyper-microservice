use std::{
    env, fmt,
    sync::{Arc, Mutex},
};

use futures::{future, Future};
use hyper::{service::service_fn, Body, Method, Request, Response, Server, StatusCode};
use log::{debug, info, trace};
use slab::Slab;

type UserId = u64;
struct UserData;
type UserDb = Arc<Mutex<Slab<UserData>>>;

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

const USER_PATH: &str = "/user/";

impl fmt::Display for UserData {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("{}")
    }
}

fn main() {
    pretty_env_logger::init();
    info!("Random Microservice - v0.1.0");

    trace!("Starting up...");
    // let addr = ([0, 0, 0, 0], 8086).into();
    let addr = env::var("ADDR")
        .unwrap_or_else(|_| "0.0.0.0:8086".into())
        .parse()
        .expect("Can't parse ADDR");

    debug!("Trying to bind server to {}", addr);
    let builder = Server::bind(&addr);
    trace!("Creating service handler");

    let user_db = Arc::new(Mutex::new(Slab::new()));
    let server = builder.serve(move || {
        let user_db = user_db.clone();
        service_fn(move |req| microservice_handler(req, &user_db))
    });

    info!("Used address {}", addr);
    let server = server.map_err(drop);

    debug!("Run!");
    hyper::rt::run(server);
}

fn response_with_code(code: StatusCode) -> Response<Body> {
    Response::builder()
        .status(code)
        .body(Body::empty())
        .unwrap()
}

fn microservice_handler(
    req: Request<Body>,
    user_db: &UserDb,
) -> impl Future<Item = Response<Body>, Error = hyper::Error> {
    let response = {
        match (req.method(), req.uri().path()) {
            (&Method::GET, "/") => Response::new(INDEX.into()),

            (&Method::GET, "/rnd") => {
                let random_bytes = rand::random::<u8>();
                Response::new(Body::from(random_bytes.to_string()))
            }

            (method, path) if path.starts_with(USER_PATH) => {
                let user_id = path
                    .trim_start_matches(USER_PATH)
                    .parse::<UserId>()
                    .ok()
                    .map(|x| x as usize);
                let mut users = user_db.lock().unwrap();
                match (method, user_id) {
                    (&Method::POST, None) => {
                        let id = users.insert(UserData);
                        Response::new(id.to_string().into())
                    }
                    (&Method::POST, Some(_)) => response_with_code(StatusCode::BAD_REQUEST),
                    (&Method::GET, Some(id)) => {
                        if let Some(data) = users.get(id) {
                            Response::new(data.to_string().into())
                        } else {
                            response_with_code(StatusCode::NOT_FOUND)
                        }
                    }
                    (&Method::PUT, Some(id)) => {
                        if let Some(user) = users.get_mut(id) {
                            *user = UserData;
                            response_with_code(StatusCode::OK)
                        } else {
                            response_with_code(StatusCode::NOT_FOUND)
                        }
                    }
                    (&Method::DELETE, Some(id)) => {
                        if users.contains(id) {
                            users.remove(id);
                            response_with_code(StatusCode::OK)
                        } else {
                            response_with_code(StatusCode::NOT_FOUND)
                        }
                    }
                    _ => response_with_code(StatusCode::METHOD_NOT_ALLOWED),
                }
            }

            _ => response_with_code(StatusCode::NOT_FOUND),
        }
    };
    future::ok(response)
}
