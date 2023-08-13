use std::sync::Mutex;

use socks::{HttpRequest, HttpResponse};
use socks_macro::{endpoint, model};

#[derive(Debug)]
#[model("/users" name password description > id name description)]
pub struct User {
    pub id: u64,
    pub name: String,
    pub password: String,
    pub description: String,
}

impl User {
    fn from_request(request: &str, id: u64) -> Option<Self> {
        let json = socks::map_json(request)?;

        Some(User {
            id,
            name: json["name"].parse().ok()?,
            password: json["password"].parse().ok()?,
            description: json["description"].parse().ok()?,
        })
    }

    fn to_response(&self) -> String {
        format!(
            "{{\"id\":{},\"name\":\"{}\",\"description\":\"{}\"}}",
            self.id, self.name, self.description
        )
    }
}

static USERS: Mutex<Vec<User>> = Mutex::new(Vec::new());

#[endpoint(POST "/users")]
fn create_user(request: HttpRequest) -> HttpResponse {
    let mut users = USERS.lock().unwrap();
    match User::from_request(&request.body.unwrap(), users.len() as u64) {
        Some(user) => {
            println!("{:?}", user);
            users.push(user);
            HttpResponse::Ok("User created!".to_string())
        }
        None => HttpResponse::BadRequest,
    }
}

#[endpoint(GET "/users")]
fn get_users(request: HttpRequest) -> HttpResponse {
    let users = USERS.lock().unwrap();

    let mut json = "[".to_string();
    for (i, user) in users.iter().enumerate() {
        json += &user.to_response();

        if i != users.len() - 1 {
            json += ",";
        }
    }
    json += "]";

    HttpResponse::Json(json)
}

#[endpoint(GET "/users/{id}")]
fn get_user(request: HttpRequest) -> HttpResponse {
    match request.url[7..].parse::<u64>() {
        Ok(id) => {
            let users = USERS.lock().unwrap();

            match users.iter().find(|e| e.id == id) {
                Some(user) => HttpResponse::Json(user.to_response()),
                None => HttpResponse::NotFound,
            }
        }
        Err(_) => {
            return HttpResponse::BadRequest;
        }
    }
}

fn main() {
    let routes = [create_user_route, get_users_route, get_user_route];
    socks::run("127.0.0.1:8080", &routes);
}
