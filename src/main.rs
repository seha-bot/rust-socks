use std::{collections::HashMap, sync::Mutex};

use socks::{HttpRequest, HttpResponse};
use socks_macro::endpoint;

static MESSAGES: Mutex<Vec<String>> = Mutex::new(Vec::new());
static NAMES: Mutex<Option<HashMap<String, String>>> = Mutex::new(None);

#[endpoint(GET "/messages")]
fn get_messages(request: HttpRequest) -> HttpResponse {
    let mut messages = MESSAGES.lock().unwrap();
    let messages = messages.join("");
    HttpResponse::Ok(messages)
}

#[endpoint(POST "/messages")]
fn post_message(request: HttpRequest) -> HttpResponse {
    let mut messages = MESSAGES.lock().unwrap();
    let names = NAMES.lock().unwrap();
    let names = names.as_ref().unwrap();

    let name = match names.get(&request.caller_ip) {
        Some(name) => name.clone(),
        None => request.caller_ip,
    };

    messages.push(format!(
        "<p>{name}</p><div>{}</div>",
        request.body.unwrap().replace("\\n", "<br>")
    ));

    HttpResponse::Ok(String::new())
}

#[endpoint(GET "/rename/{name}")]
fn rename(request: HttpRequest) -> HttpResponse {
    let mut names = NAMES.lock().unwrap();
    let names = names.as_mut().unwrap();

    let name = &request.url[8..];
    names.insert(request.caller_ip.to_string(), name.to_string());

    HttpResponse::Ok(String::new())
}

fn main() {
    {
        let mut names = NAMES.lock().unwrap();
        *names = Some(HashMap::from([(
            String::from("127.0.0.1"),
            String::from("MASTER"),
        )]));
    }

    let routes = [get_messages_route, post_message_route, rename_route];
    socks::run("0.0.0.0:8080", &routes);
}
