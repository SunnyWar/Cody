// src/main.rs
#![allow(non_snake_case)]

mod api;

fn main() {
    let mut api = api::uciapi::CodyApi::new();
    api.run();
}
