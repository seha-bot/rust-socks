use socks::{HttpRequest, HttpResponse, Route};

fn index(_request: HttpRequest) -> HttpResponse {
    HttpResponse::Ok("I am alive!!".to_string())
}

fn bob(request: HttpRequest) -> HttpResponse {
    let msg = request.url()[1];
    HttpResponse::Ok(format!("Bob is {}", msg))
}

fn djejson(request: HttpRequest) -> HttpResponse {
    println!("{:?}", request.body().unwrap());
    HttpResponse::Ok(format!("This is some good data :)"))
}

fn main() {
    let routes = [
        Route {
            request: HttpRequest::Get("/".to_string()),
            handler: index,
        },
        Route {
            request: HttpRequest::Get("/bob/{msg}".to_string()),
            handler: bob,
        },
        Route {
            request: HttpRequest::Post {
                url: "/bob".to_string(),
                body: String::new(),
            },
            handler: djejson,
        },
    ];
    socks::run(&routes);
}
