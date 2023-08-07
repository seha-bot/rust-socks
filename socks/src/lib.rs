use std::fs::{self, File};
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::time::Duration;

pub enum HttpRequest {
    Get(String),
    Post { url: String, body: String },
}

impl HttpRequest {
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

        let words: Vec<&str> = content.split_whitespace().collect();
        let req_type = *words.get(0)?;
        let url = words.get(1)?.to_string();

        match req_type {
            "GET" => Some(HttpRequest::Get(url)),
            "POST" => {
                let len_start = content.find("Content-Length:")? + 16;
                let len_end = len_start + content[len_start..].find('\r')?;

                match content[len_start..len_end].parse::<usize>() {
                    Ok(len) => Some(HttpRequest::Post {
                        url,
                        body: content[len_end + 4..len_end + 4 + len].to_string(),
                    }),
                    Err(_) => {
                        return None;
                    }
                }
            }
            _ => None,
        }
    }

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
}

pub enum HttpResponse {
    Ok(String),
    BadRequest,
    NotFound,
    ServerError,
}

impl HttpResponse {
    fn as_bytes(&self) -> Vec<u8> {
        match self {
            HttpResponse::Ok(msg) => format!("HTTP/1.0 200 OK\r\n\r\n{msg}").as_bytes().to_vec(),
            HttpResponse::BadRequest => "HTTP/1.0 400\r\n\r\n".as_bytes().to_vec(),
            HttpResponse::NotFound => "HTTP/1.0 404\r\n\r\n".as_bytes().to_vec(),
            HttpResponse::ServerError => "HTTP/1.0 500\r\n\r\n".as_bytes().to_vec(),
        }
    }
}

pub struct Route {
    pub request: HttpRequest,
    pub handler: Box<dyn Fn(HttpRequest) -> HttpResponse>,
}

impl Route {
    fn from_file(request_path: String, file_path: String) -> Self {
        Route {
            request: HttpRequest::Get(request_path),
            handler: Box::new(move |_| {
                let mut file = match File::open(&file_path) {
                    Ok(val) => val,
                    Err(_) => {
                        return HttpResponse::ServerError;
                    }
                };

                let mut content = String::new();
                if let Err(_) = file.read_to_string(&mut content) {
                    return HttpResponse::ServerError;
                }

                HttpResponse::Ok(content)
            }),
        }
    }

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

fn handle_client(routes: &Vec<Route>, mut stream: TcpStream) {
    if let Some(request) = HttpRequest::from_stream(&mut stream) {
        let mut response = match request {
            HttpRequest::Get(_) => HttpResponse::Ok("404 Nothing here :/".to_string()),
            HttpRequest::Post { url: _, body: _ } => HttpResponse::BadRequest,
        };

        if let Some(route) = routes.iter().find(|e| e.is_basically_the_same_as(&request)) {
            response = (route.handler)(request);
        }

        let _ = stream.write(&response.as_bytes());
    }
}

fn add_all_dirs(routes: &mut Vec<Route>, path: &str) {
    if let Ok(paths) = fs::read_dir(path) {
        for path in paths {
            if let Ok(path) = path {
                if let Ok(data) = path.metadata() {
                    if data.is_dir() {
                        add_all_dirs(routes, path.path().to_str().unwrap());
                        continue;
                    }

                    let file_path = path.path().to_str().unwrap().to_string();
                    let request_path = file_path.replace("./www", "");

                    if file_path.ends_with("index.html") {
                        routes.push(Route::from_file(
                            request_path.replace("index.html", ""),
                            file_path.clone(),
                        ));
                    }

                    routes.push(Route::from_file(request_path, file_path));
                }
            }
        }
    }
}

pub fn run(routes: &[fn() -> Route]) {
    let mut routes: Vec<Route> = routes.iter().map(|e| e()).collect();

    add_all_dirs(&mut routes, "./www");

    let listener = TcpListener::bind("127.0.0.1:8080").unwrap();

    for stream in listener.incoming() {
        if stream.is_err() {
            continue;
        }
        handle_client(&routes, stream.unwrap());
    }
}
