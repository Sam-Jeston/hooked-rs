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
use rocket::State;
use rocket_contrib::json::Json;
use std::fs;
use queue::Queue;
use std::sync::Arc;
use std::{thread, time};

#[get("/targets")]
fn targets(targets: State<Vec<config::Target>>) -> Json<Vec<config::Target>> {
    Json(targets.to_vec())
}

#[post("/", format = "json", data = "<hook>")]
fn hook(targets: State<Vec<config::Target>>, queue: State<Arc<Queue>>, hook: Json<StatusPayload>) {
    handler::process_payload(queue, hook, targets).unwrap();
}

fn main() {
    let yml = fs::read_to_string("example.yml").expect("No yaml configuration provided");

    let config = config::parse_config(yml).expect("yaml failed to parse, better check its format");
    let queue = Arc::new(Queue::new());

    // TODO Use programatic ConfigBuilder to respect Port set in config
    let server_queue_ref = Arc::clone(&queue);
    let server = thread::spawn(move || {
        rocket::ignite()
            .manage(config.targets)
            .manage(server_queue_ref)
            .mount("/", routes![targets, hook])
            .launch();
    });

    let worker_queue_ref = Arc::clone(&queue);
    let worker = thread::spawn(move || {
        loop {
            // Restart the queue every five second. The queue
            // will empty itself, so if it is backed up, this
            // does nothing
            worker_queue_ref.process().unwrap();
            let one_second = time::Duration::from_millis(5000);
            thread::sleep(one_second);
        }
    });

    server.join().expect("failed to start server");
    worker.join().expect("failed to start worker");
}
