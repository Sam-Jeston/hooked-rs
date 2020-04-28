#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use] extern crate rocket;
extern crate serde;
extern crate serde_yaml;

mod builder;
mod hook_handler;

use std::fs;
use rocket_contrib::json::Json;
use rocket::State;

#[get("/builds")]
fn builds(builds: State<Vec<builder::Build>>) -> Json<Vec<builder::Build>> {
    Json(builds.to_vec())
}

fn main() {
    let yml = fs::read_to_string("example.yml")
        .expect("No yaml configuration provided");
    
    let builds = builder::parse_builds(yml)
        .expect("yaml failed to parse, better check its format");

    rocket::ignite()
        .manage(builds)
        .mount(
            "/",
            routes![builds]
        ).launch();
}