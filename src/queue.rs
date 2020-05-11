use super::job::Job;
use duct::cmd;
use log::{error, info};
use std::collections::VecDeque;
use std::io::prelude::*;
use std::io::BufReader;
use std::sync::Mutex;

pub struct Queue {
    pub jobs: Mutex<VecDeque<Job>>,
}

impl Queue {
    pub fn new() -> Queue {
        Queue {
            jobs: Mutex::new(VecDeque::new()),
        }
    }

    pub fn add(&self, job: Job) -> Result<(), &str> {
        match self.jobs.lock() {
            Ok(mut jobs) => Ok(jobs.push_back(job)),
            Err(_) => Err("Failed to read queue length"),
        }
    }

    pub fn length(&self) -> Result<usize, &str> {
        match self.jobs.lock() {
            Ok(jobs) => Ok(jobs.len()),
            Err(_) => Err("Failed to add job to queue"),
        }
    }

    pub fn get_job(&self) -> Option<Job> {
        // Only hold the lock for as short a time as possible,
        // which is just to get the most recent job and release the lock
        match self.jobs.try_lock() {
            Ok(mut jobs) => jobs.pop_front(),
            _ => None,
        }
    }

    pub fn process(&self) -> Result<(), Job> {
        let job = &self.get_job();
        match job {
            None => Ok(()),
            Some(j) => {
                info!("Processing job for {}", j.id);
                let cmd = cmd!("sh", "-c", &j.command).stderr_to_stdout().reader();

                match cmd {
                    Ok(reader) => {
                        let lines = BufReader::new(reader).lines();
                        for line in lines {
                            match line {
                                Ok(l) => info!("{}", l),
                                // This is a logging utility. An error here is not great,
                                // but not worth panicing over
                                Err(_) => error!("Error when reading process lines"),
                            }
                        }

                        self.process()
                    }
                    Err(_) => Err(j.clone()),
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_add_to_queue() {
        let queue = Queue::new();
        let job = Job {
            id: "x".to_owned(),
            command: format!("cd /var/www;ls stat;echo hello;"),
        };
        queue.add(job.clone()).unwrap();
        let length = queue.length().unwrap();
        assert_eq!(length, 1);

        queue.add(job.clone()).unwrap();
        let additional_length = queue.length().unwrap();
        assert_eq!(additional_length, 2);
    }

    #[test]
    fn can_get_job() {
        let queue = Queue::new();
        let job = Job {
            id: "x".to_owned(),
            command: format!("cd /var/www;ls stat;echo hello;"),
        };
        queue.add(job.clone()).unwrap();

        let job_from_queue = queue.get_job().unwrap();
        assert_eq!(job_from_queue, job);

        let queue_length = queue.length().unwrap();
        assert_eq!(queue_length, 0);
    }

    #[test]
    fn gets_jobs_in_fifo_order() {
        let queue = Queue::new();
        let job1 = Job {
            id: "x".to_owned(),
            command: format!("cd /var/xyz;"),
        };
        let job2 = Job {
            id: "y".to_owned(),
            command: format!("cd /var/www;"),
        };
        queue.add(job1.clone()).unwrap();
        queue.add(job2.clone()).unwrap();

        let job_from_queue = queue.get_job().unwrap();
        assert_eq!(job_from_queue, job1);

        let job_from_queue = queue.get_job().unwrap();
        assert_eq!(job_from_queue, job2);

        let queue_length = queue.length().unwrap();
        assert_eq!(queue_length, 0);
    }

    #[test]
    fn can_process_empties_queue() {
        let queue = Queue::new();
        let job = Job {
            id: "x".to_owned(),
            command: format!("echo \"hello world!\";"),
        };

        // Add two jobs to the queue
        queue.add(job.clone()).unwrap();
        queue.add(job.clone()).unwrap();

        queue.process().unwrap();

        let queue_length = queue.length().unwrap();
        assert_eq!(queue_length, 0);
    }
}
