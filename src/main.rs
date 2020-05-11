#![feature(proc_macro_hygiene, decl_macro, with_options)]

#[macro_use]
extern crate rocket;
extern crate log;
extern crate serde;
extern crate serde_yaml;
extern crate simplelog;
extern crate base64;
extern crate rand;
extern crate duct;

mod config;
mod handler;
mod job;
mod queue;

use handler::StatusPayload;
use queue::Queue;
use rocket::config::{Config, Environment, LoggingLevel};
use rocket::State;
use rocket_contrib::json::Json;
use simplelog::{TermLogger, CombinedLogger, WriteLogger, LevelFilter, TerminalMode, Config as LogConfig};
use std::env;
use std::fs;
use std::sync::Arc;
use std::{thread, time};
use std::fs::File;
use log::info;
use rand::{Rng};
use base64::{encode};
use rocket::http::Status;

#[post("/", format = "json", data = "<hook>")]
fn hook(targets: State<Vec<config::Target>>, queue: State<Arc<Queue>>, hook: Json<StatusPayload>) -> Status {
    handler::process_payload(queue, hook, targets)
}

fn main() {
    // Determine path to config from args, and parse the file
    // Panic if any of these steps fail.
    let args: Vec<String> = env::args().collect();
    let yml_path = config::parse_args(args).expect("--config <path> must be provided");
    let yml = fs::read_to_string(yml_path).expect("Config file does not exist");
    let config =
        config::parse_config(yml).expect("Config failed to parse, better check its format");

    // Initialise the logger
    // A new log file is created if it doesnt already exist
    let config_file_path = config.log.clone();
    let log_file = File::with_options().append(true).create(true).open(&config_file_path).unwrap();
    CombinedLogger::init(
        vec![
            TermLogger::new(LevelFilter::Info, LogConfig::default(), TerminalMode::Mixed).unwrap(),
            WriteLogger::new(LevelFilter::Info, LogConfig::default(), log_file),
        ]
    ).unwrap();

    info!("hooked-rs starting with config:");
    info!("{:?}", config);

    let queue = Arc::new(Queue::new());
    let server_queue_ref = Arc::clone(&queue);
    let server = thread::spawn(move || {
        // Generate entropy for rocket.rs secret to prevent warnings. We don't
        // use this value.
        let random_bytes = rand::thread_rng().gen::<[u8; 32]>();

        let server_config = Config::build(Environment::Production)
            .address(config.host)
            .port(config.port)
            .log_level(LoggingLevel::Off)
            .secret_key(encode(random_bytes))
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
            let five_seconds = time::Duration::from_millis(5000);
            thread::sleep(five_seconds);
        }
    });

    server.join().expect("Failed to start API server");
    worker.join().expect("Failed to start queue processor");
}
