use proc_macro::TokenStream;

#[proc_macro_attribute]
pub fn endpoint(args: TokenStream, mut item: TokenStream) -> TokenStream {
    let args = args.to_string();
    let args: Vec<&str> = args.split_whitespace().collect();

    let mut item_iter = item.clone().into_iter();
    item_iter.find(|e| e.to_string() == "fn");
    let name = item_iter.nth(0).unwrap().to_string();

    let route = format!(
        "fn {name}_route()->socks::Route{{socks::Route::new(String::from({}),String::from(\"{}\"),Box::new({name}))}}",
        args[1], args[0]
    );

    item.extend(route.parse::<TokenStream>().unwrap());
    item
}
