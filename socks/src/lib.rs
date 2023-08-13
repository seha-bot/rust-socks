use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::time::Duration;

pub struct HttpRequest {
    pub url: String,
    pub verb: String,
    pub caller_ip: String,
    pub body: Option<String>,
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
            content.push_str(String::from_utf8(buffer.to_vec()).ok()?.trim_matches('\0'));
            if buffer[1023] == 0 {
                break;
            }
        }

        let content: String = content.chars().filter(|c| *c != '\r').collect();
        let lines: Vec<&str> = content.split('\n').collect();

        let headers: HashMap<&str, &str> = lines[1..]
            .iter()
            .filter_map(|e| {
                let e = e.split_once(':')?;
                Some((e.0.trim(), e.1.trim()))
            })
            .collect();

        let words: Vec<&str> = lines.get(0)?.split_whitespace().collect();
        let verb = words.get(0)?.to_string();

        Some(HttpRequest {
            url: words.get(1)?.to_string(),
            caller_ip: stream.peer_addr().ok()?.ip().to_string(),
            body: if verb == "POST" {
                let body_start = content.find("\n\n")? + 2;
                let len: usize = headers.get("Content-Length")?.parse().ok()?;
                Some(content.get(body_start..body_start + len)?.to_string())
            } else {
                None
            },
            verb,
        })
    }
}

pub enum HttpResponse {
    Ok(String),
    Json(String),
    Raw(Vec<u8>),
    BadRequest,
    NotFound,
    ServerError,
}

impl HttpResponse {
    fn as_bytes(&self) -> Vec<u8> {
        let mut http = "HTTP/1.1 ".as_bytes().to_vec();

        http.extend(match self {
            HttpResponse::Ok(msg) => format!("200\r\n\r\n{msg}").into_bytes(),
            HttpResponse::Json(json) => {
                format!("200\r\nContent-Type: application/json\r\n\r\n{json}").into_bytes()
            }
            HttpResponse::Raw(bytes) => {
                let mut headers =
                    format!("200\r\nContent-Length: {}\r\n\r\n", bytes.len()).into_bytes();

                headers.extend(bytes);
                headers
            }
            HttpResponse::BadRequest => "400\r\n\r\n".as_bytes().to_vec(),
            HttpResponse::NotFound => "404\r\n\r\n".as_bytes().to_vec(),
            HttpResponse::ServerError => "500\r\n\r\n".as_bytes().to_vec(),
        });

        http
    }
}

pub struct Route {
    pub request: HttpRequest,
    pub handler: Box<dyn Fn(HttpRequest) -> HttpResponse>,
}

impl Route {
    pub fn new(
        url: String,
        verb: String,
        handler: Box<dyn Fn(HttpRequest) -> HttpResponse>,
    ) -> Self {
        Route {
            request: HttpRequest {
                url,
                verb,
                caller_ip: String::new(),
                body: None,
            },
            handler,
        }
    }

    fn from_file(request_path: String, file_path: String) -> Self {
        Self::new(
            request_path,
            String::from("GET"),
            Box::new(move |_| match File::open(&file_path) {
                Ok(mut file) => {
                    let mut content = Vec::<u8>::new();
                    if let Err(_) = file.read_to_end(&mut content) {
                        return HttpResponse::ServerError;
                    }

                    HttpResponse::Raw(content)
                }
                Err(_) => HttpResponse::ServerError,
            }),
        )
    }

    fn is_basically_the_same_as(&self, request: &HttpRequest) -> bool {
        if self.request.url.is_empty() && request.url == "/" {
            return true;
        } else if self.request.verb != request.verb {
            return false;
        }

        let self_dirs = self.request.url.split('/');
        let dirs = request.url.split('/');

        for (dir, self_dir) in dirs.clone().zip(self_dirs.clone()) {
            if dir != self_dir && !(self_dir.starts_with('{') && self_dir.ends_with('}')) {
                return false;
            }
        }

        dirs.count() == self_dirs.count()
    }
}

pub fn map_json(json: &str) -> Option<HashMap<&str, &str>> {
    if json.chars().nth(0)? != '{' {
        return None;
    }

    let mut map = HashMap::new();
    let mut value_start = 1;
    let mut last_key = None;
    let mut is_in_quotes = false;

    for i in 0..json.len() {
        match (json.chars().nth(i)?, is_in_quotes) {
            ('"', _) => {
                if json.chars().nth(i - 1)? != '\\' {
                    is_in_quotes = !is_in_quotes;
                }
            }
            (':', false) => {
                last_key = Some(&json[value_start + 1..i - 1]);
                value_start = i + 1;
            }
            (',', false) | ('}', false) => {
                if json.chars().nth(value_start)? == '"' && json.chars().nth(i - 1)? == '"' {
                    map.insert(last_key?, &json[value_start + 1..i - 1]);
                } else {
                    map.insert(last_key?, &json[value_start..i]);
                }
                last_key = None;
                value_start = i + 1;
            }
            _ => (),
        };
    }

    Some(map)
}

fn add_all_dirs(routes: &mut Vec<Route>, path: &str) {
    if let Ok(paths) = fs::read_dir(path) {
        for path in paths {
            if let Ok(path) = path {
                if let Ok(data) = path.metadata() {
                    let path = path.path().to_str().unwrap().to_string();
                    let request_path = path.replace("www", "");

                    if data.is_dir() {
                        add_all_dirs(routes, &path);
                        continue;
                    }

                    if path.ends_with("index.html") {
                        routes.push(Route::from_file(
                            request_path.replace("/index.html", ""),
                            path.clone(),
                        ));
                    }

                    routes.push(Route::from_file(request_path, path));
                }
            }
        }
    }
}

fn handle_client(routes: &Vec<Route>, mut stream: TcpStream) {
    if let Some(request) = HttpRequest::from_stream(&mut stream) {
        let mut response = match request.verb.as_str() {
            "GET" => HttpResponse::Ok("404 Nothing here :/".to_string()),
            _ => HttpResponse::BadRequest,
        };

        if let Some(route) = routes.iter().find(|e| e.is_basically_the_same_as(&request)) {
            response = (route.handler)(request);
        }

        let _ = stream.write(&response.as_bytes());
    }
}

pub fn run(address: &str, routes: &[fn() -> Route]) {
    let mut routes: Vec<Route> = routes.iter().map(|e| e()).collect();
    add_all_dirs(&mut routes, "www");

    let listener = TcpListener::bind(address).unwrap();

    for stream in listener.incoming() {
        if stream.is_err() {
            continue;
        }
        handle_client(&routes, stream.unwrap());
    }
}
