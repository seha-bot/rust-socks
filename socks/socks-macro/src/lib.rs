use proc_macro::TokenStream;

#[proc_macro_attribute]
pub fn endpoint(args: TokenStream, item: TokenStream) -> TokenStream {
    let item = item.to_string();
    let args = args.to_string();
    let args: Vec<&str> = args.split(' ').collect();

    let name = item[item.find("fn").unwrap() + 3..]
        .split('(')
        .nth(0)
        .unwrap();

    let request = match args[0] {
        "GET" => format!("HttpRequest::Get({}.to_string())", args[1]),
        "POST" => format!(
            "HttpRequest::Post{{url:{}.to_string(),body:String::new()}}",
            args[1]
        ),
        _ => panic!(),
    };

    let route =
        format!("fn {name}_route()->Route{{Route{{request:{request},handler:Box::new({name})}}}}");
    (item + &route).parse().unwrap()
}

#[proc_macro_attribute]
pub fn model(args: TokenStream, item: TokenStream) -> TokenStream {
    let item = item.to_string();
    let args = args.to_string();
    let _args: Vec<&str> = args.split(' ').collect();

    let start = item.find('{').unwrap();
    let name = item[7..start - 1].trim();

    let mut model = Vec::<&str>::new();
    let mut requests = Vec::<&str>::new();
    let mut responses = Vec::<&str>::new();
    let mut current_sort = None;

    for word in item[start + 1..item.len() - 1].split_whitespace() {
        match word {
            "#[req]" | "#[res]" | "#[all]" => {
                current_sort = Some(word);
                continue;
            }
            _ => (),
        }

        model.push(word);

        match current_sort {
            Some("#[req]") => {
                requests.push(word);
            }
            Some("#[res]") => {
                responses.push(word);
            }
            Some("#[all]") => {
                requests.push(word);
                responses.push(word);
            }
            _ => (),
        }
    }

    let model = model.join("");
    let _requests = requests.join("");
    let _responses = responses.join("");

    format!("pub struct {name} {{ {model} }}").parse().unwrap()
}
