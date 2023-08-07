use socks::{HttpRequest, HttpResponse, Route};
use socks_macro::endpoint;

#[endpoint(GET "/")]
fn index(_request: HttpRequest) -> HttpResponse {
    HttpResponse::Ok("I am alive!!".to_string())
}

#[endpoint(GET "/bob/{msg}")]
fn bob(request: HttpRequest) -> HttpResponse {
    HttpResponse::Ok(format!("Bob says {}", request.url()[1]))
}

#[endpoint(POST "/bob")]
fn bob_post(request: HttpRequest) -> HttpResponse {
    println!("REQUEST {}", request.body().unwrap());
    HttpResponse::Ok("Thank you for this data :)".to_string())
}

fn main() {
    let routes = [index_route, bob_route, bob_post_route];
    socks::run(&routes);
}
