use socks::{HttpRequest, HttpResponse, Route};
use socks_macro::endpoint;

// fn index_body(_request: HttpRequest) -> HttpResponse {
//     HttpResponse::Ok("I am alive!!".to_string())
// }
//
// fn index() -> Route {
//     Route {
//         request: HttpRequest::Get("/".to_string()),
//         handler: index_body,
//     }
// }
//
// fn bob_body(request: HttpRequest) -> HttpResponse {
//     let msg = request.url()[1];
//     HttpResponse::Ok(format!("Bob is {}", msg))
// }
//
// fn bob() -> Route {
//     Route {
//         request: HttpRequest::Get("/bob/{msg}".to_string()),
//         handler: bob_body,
//     }
// }
//
// fn djejson_body(request: HttpRequest) -> HttpResponse {
//     println!("{:?}", request.body().unwrap());
//     HttpResponse::Ok(format!("This is some good data :)"))
// }
//
// fn djejson() -> Route {
//     Route {
//         request: HttpRequest::Post {
//             url: "/bob/".to_string(),
//             body: String::new(),
//         },
//         handler: djejson_body,
//     }
// }

#[endpoint(GET "/")]
fn index(_request: HttpRequest) -> HttpResponse {
    HttpResponse::Ok("I am alive!!".to_string())
}

fn main() {
    let routes: [fn() -> Route; 1] = [index];
    socks::run(&routes);
}
