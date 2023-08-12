use proc_macro::TokenStream;

#[proc_macro_attribute]
pub fn endpoint(args: TokenStream, item: TokenStream) -> TokenStream {
    let item = item.to_string();
    let args = args.to_string();
    let args: Vec<&str> = args.split(' ').collect();

    let name = item[3..].split('(').nth(0).unwrap();
    let route = format!(
        "fn {name}_route()->Route{{Route::new(String::from({}),String::from(\"{}\"),Box::new({name}))}}",
        args[1], args[0]
    );

    (item + &route).parse().unwrap()
}
