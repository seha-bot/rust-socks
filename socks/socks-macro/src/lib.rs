use proc_macro::TokenStream;

#[proc_macro_attribute]
pub fn endpoint(args: TokenStream, item: TokenStream) -> TokenStream {
    let item = item.to_string();
    let args = args.to_string();
    let args: Vec<&str> = args.split_whitespace().collect();

    let name = item[item.find("fn").unwrap() + 3..]
        .split('(')
        .nth(0)
        .unwrap()
        .trim();

    let route = format!(
        "fn {name}_route()->socks::Route{{socks::Route::new(String::from({}),String::from(\"{}\"),Box::new({name}))}}",
        args[1], args[0]
    );

    (item + &route).parse().unwrap()
}

#[proc_macro_attribute]
pub fn model(args: TokenStream, item: TokenStream) -> TokenStream {
    let args = args.to_string();
    let args: Vec<&str> = args.split_whitespace().collect();

    let mut tokens = item.clone().into_iter();

    let name = match tokens.nth(1).unwrap().to_string().as_str() {
        "struct" => tokens.nth(0).unwrap().to_string(),
        name => name.to_string(),
    };

    let body = tokens.nth(0).unwrap().to_string().replace("pub ", "");
    let body = body.split_whitespace().collect::<String>();
    let body = body[1..body.len() - 1]
        .split(',')
        .map(|e| e.split_once(':').unwrap());

    let mut sort = (String::new(), String::new());

    for &arg in args[1..].iter() {
        if arg == ">" {
            sort = (sort.1, sort.0);
        } else if let Some(param) = body.clone().filter(|e| e.0 == arg).nth(0) {
            sort.0 += &format!("{}:{},", param.0, param.1);
        }
    }

    let documentation = stringify! {
        impl socks::Documentation for User {
            fn name() -> &'static str {
                "documentation_name"
            }

            fn endpoint() -> &'static str {
                documentation_endpoint
            }

            fn request() -> &'static str {
                "documentation_request"
            }

            fn response() -> &'static str {
                "documentation_response"
            }
        }
    };

    let documentation = documentation
        .replace("documentation_name", &name)
        .replace("documentation_endpoint", args[0])
        .replace("documentation_request", &sort.1)
        .replace("documentation_response", &sort.0);

    (item.to_string() + &documentation).parse().unwrap()
}
