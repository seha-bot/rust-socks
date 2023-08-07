use socks::{HttpRequest, HttpResponse, Route};
use socks_macro::endpoint;

#[endpoint(GET "/")]
fn index(_request: HttpRequest) -> HttpResponse {
    HttpResponse::Ok("I am alive!!".to_string())
}

fn main() {
    let routes: [fn() -> Route; 1] = [index_route];
    socks::run(&routes);
}
