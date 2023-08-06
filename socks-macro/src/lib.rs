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

    let item = item.replace(
        format!("fn {name}").as_str(),
        format!("fn {name}_body").as_str(),
    );

    let request = match args[0] {
        "GET" => format!("HttpRequest::Get({}.to_string())", args[1]),
        _ => panic!(),
    };

    let wrapper = format!("fn {name}()->Route{{Route{{request:{request},handler:{name}_body}}}}");
    (item + &wrapper).parse().unwrap()
}
