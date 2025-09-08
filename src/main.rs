#![allow(non_snake_case)]

mod api; // bring in your new api.rs

fn main() {
    let mut api = api::CodyApi::new();
    api.run();
}
