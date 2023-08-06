use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::time::Duration;

pub struct Route {
    pub request: HttpRequest,
    pub handler: fn(HttpRequest) -> HttpResponse,
}

impl Route {
    fn is_basically_the_same_as(&self, request: &HttpRequest) -> bool {
        let self_dirs = self.request.url();
        let dirs = request.url();

        if self_dirs.len() != dirs.len()
            || std::mem::discriminant(&self.request) != std::mem::discriminant(request)
        {
            return false;
        }

        for (i, v) in dirs.iter().enumerate() {
            let mut chars = self_dirs[i].chars();

            if chars.nth(0) == Some('{') && chars.nth_back(0) == Some('}') {
                continue;
            }
            if *v != self_dirs[i] {
                return false;
            }
        }
        true
    }
}

pub enum HttpResponse {
    Ok(String),
    BadRequest,
    NotFound,
}

impl HttpResponse {
    fn as_bytes(&self) -> Vec<u8> {
        match self {
            HttpResponse::Ok(msg) => format!("HTTP/1.0 200 OK\r\n\r\n{msg}").as_bytes().to_vec(),
            HttpResponse::BadRequest => "HTTP/1.0 400\r\n\r\n".as_bytes().to_vec(),
            HttpResponse::NotFound => "HTTP/1.0 404\r\n\r\n".as_bytes().to_vec(),
        }
    }
}

#[derive(Eq, PartialEq)]
pub enum HttpRequest {
    Get(String),
    Post { url: String, body: String },
}

impl HttpRequest {
    pub fn url(&self) -> Vec<&str> {
        match self {
            HttpRequest::Get(url) => url,
            HttpRequest::Post { url, body: _ } => url,
        }
        .split('/')
        .filter(|e| !e.is_empty())
        .collect()
    }

    pub fn body(&self) -> Option<String> {
        match self {
            HttpRequest::Post { url: _, body } => Some(body.clone()),
            _ => None,
        }
    }

    fn from_string(content: String) -> Option<Self> {
        let words: Vec<&str> = content.split_whitespace().collect();

        let req_type = *words.get(0)?;
        let url = words.get(1)?.to_string();

        match req_type {
            "GET" => Some(HttpRequest::Get(url)),
            "POST" => {
                let i = content.find("Content-Length:");
                if i.is_none() {
                    return None;
                }

                let len_start = i.unwrap() + 16;
                let len_end = len_start + content[len_start..].find('\r')?;

                let len: usize = match content[len_start..len_end].parse() {
                    Ok(val) => val,
                    Err(_) => {
                        return None;
                    }
                };

                Some(HttpRequest::Post {
                    url,
                    body: content[len_end + 4..len_end + 4 + len].to_string(),
                })
            }
            _ => None,
        }
    }

    fn from_stream(stream: &mut TcpStream) -> Option<Self> {
        let _ = stream.set_read_timeout(Some(Duration::from_millis(100)));
        let mut content = String::new();
        let mut buffer: [u8; 1024];

        loop {
            buffer = [0; 1024];
            if let Err(_) = stream.read(&mut buffer) {
                break;
            }
            content.push_str(
                String::from_utf8(buffer.to_vec())
                    .unwrap()
                    .trim_matches('\0'),
            );
            if buffer[1023] == 0 {
                break;
            }
        }

        HttpRequest::from_string(content)
    }
}

fn handle_client(routes: &[Route], mut stream: TcpStream) {
    let request = match HttpRequest::from_stream(&mut stream) {
        Some(val) => val,
        None => {
            return;
        }
    };

    let mut response = match request {
        HttpRequest::Get(_) => HttpResponse::Ok("404 Nothing here :/".to_string()),
        HttpRequest::Post { url: _, body: _ } => HttpResponse::BadRequest,
    };

    if let Some(route) = routes.iter().find(|e| e.is_basically_the_same_as(&request)) {
        response = (route.handler)(request);
    }

    let _ = stream.write(&response.as_bytes());
}

pub fn run(routes: &[Route]) {
    let listener = TcpListener::bind("127.0.0.1:8080").unwrap();

    for stream in listener.incoming() {
        if stream.is_err() {
            continue;
        }
        handle_client(routes, stream.unwrap());
    }
}
