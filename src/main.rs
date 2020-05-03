#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;
extern crate serde;
extern crate serde_yaml;

mod config;
mod handler;
mod job;
mod queue;

use handler::StatusPayload;
use queue::Queue;
use rocket::config::{Config, Environment};
use rocket::State;
use rocket_contrib::json::Json;
use std::env;
use std::fs;
use std::sync::Arc;
use std::{thread, time};

#[post("/", format = "json", data = "<hook>")]
fn hook(targets: State<Vec<config::Target>>, queue: State<Arc<Queue>>, hook: Json<StatusPayload>) {
    handler::process_payload(queue, hook, targets).unwrap();
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let yml_path = config::parse_args(args).expect("--config <path> must be provided");
    let yml = fs::read_to_string(yml_path).expect("Config file does not exist");
    let config =
        config::parse_config(yml).expect("Config failed to parse, better check its format");
    let queue = Arc::new(Queue::new());

    let server_queue_ref = Arc::clone(&queue);
    let server = thread::spawn(move || {
        let server_config = Config::build(Environment::Production)
            .address(config.host)
            .port(config.port)
            .finalize()
            .unwrap();

        rocket::custom(server_config)
            .manage(config.targets)
            .manage(server_queue_ref)
            .mount("/", routes![hook])
            .launch();
    });

    let worker_queue_ref = Arc::clone(&queue);
    let worker = thread::spawn(move || {
        loop {
            // Check the queue for jobs every five seconds. The queue
            // will process itslef recursively until it is empty.
            worker_queue_ref.process().unwrap();
            let one_second = time::Duration::from_millis(5000);
            thread::sleep(one_second);
        }
    });

    server.join().expect("Failed to start API server");
    worker.join().expect("Failed to start queue processor");
}
