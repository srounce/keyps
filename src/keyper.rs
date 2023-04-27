use itertools::Itertools;
use log::debug;
use std::{
    fs,
    path::PathBuf,
    sync::mpsc,
    thread::{self, JoinHandle},
    time::Duration,
};
use url::Url;

use crate::source::SourceIdentifier;

#[derive(Debug)]
pub struct KeyperConfig {
    pub sources: Vec<SourceIdentifier>,
    pub file_path: PathBuf,
    pub interval: u64,
}

pub struct Keyper {
    service: JoinHandle<()>,
    tx: mpsc::Sender<Message>,
    rx: mpsc::Receiver<Message>,
}

impl Keyper {
    pub fn start(config: KeyperConfig) -> Self {
        let (tx, service_rx) = mpsc::channel();
        let (service_tx, rx) = mpsc::channel();
        let timer_rx = tx.clone();

        let interval = config.interval;

        thread::spawn(move || loop {
            debug!("Do refresh");
            timer_rx.send(Message::Refresh).unwrap();

            thread::sleep(Duration::from_secs(interval));
        });

        let service = thread::spawn(move || {
            KeyperService::new(config, service_tx, service_rx).start();
        });

        Self { service, tx, rx }
    }

    pub fn stop(self) -> JoinHandle<()> {
        self.tx.send(Message::Quit).unwrap();
        self.service
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Message {
    Refresh,
    Quit,
}

struct KeyperService {
    config: KeyperConfig,
    tx: mpsc::Sender<Message>,
    rx: mpsc::Receiver<Message>,
}

impl KeyperService {
    pub fn new(
        config: KeyperConfig,
        tx: mpsc::Sender<Message>,
        rx: mpsc::Receiver<Message>,
    ) -> Self {
        Self { config, tx, rx }
    }

    pub fn start(&mut self) {
        loop {
            match self.rx.recv() {
                Ok(Message::Refresh) => self.refresh(),
                Ok(Message::Quit) => {
                    self.cleanup();
                    break;
                }
                v => debug!("KeyperService recieved: {v:#?}"),
            }
        }
    }

    fn refresh(&mut self) {
        let keys = self
            .config
            .sources
            .iter()
            .flat_map(|source| {
                let url: Url = match source {
                    SourceIdentifier::GitHub { username } => {
                        format!("https://github.com/{username}.keys")
                            .parse()
                            .unwrap()
                    }
                    SourceIdentifier::GitLab { username } => {
                        format!("https://gitlab.com/{username}.keys")
                            .parse()
                            .unwrap()
                    }
                    SourceIdentifier::Http { address } => address.clone(),
                    _ => unreachable!(),
                };
                let response = reqwest::blocking::get(url).unwrap();
                let body = response.text().unwrap();

                let lines = body.lines().map(|l| l.into()).collect::<Vec<String>>();

                lines.into_iter()
            })
            .unique()
            .collect::<Vec<_>>();

        upsert_keys(&self.config.file_path, &keys);
    }

    fn cleanup(&self) -> Result<(), ()> {
        let file_path = &self.config.file_path;
        let authorized_keys = fs::read_to_string(file_path).map_err(|_| {
            todo!("handle key update failure");
        })?;

        let mut skip_lines = false;
        let authorized_keys = authorized_keys
            .lines()
            .filter(|line| {
                let prev = skip_lines;
                skip_lines = match *line {
                    "# keyps: START" => true,
                    "# keyps: END" => false,
                    _ => skip_lines,
                };

                !(prev || skip_lines)
            })
            .collect::<Vec<_>>();

        fs::write(file_path, authorized_keys.join("\n")).map_err(|_| {
            todo!("handle key update failure");
        })?;

        Ok(())
    }
}

fn upsert_keys(file_path: &PathBuf, keys: &[String]) -> Result<(), ()> {
    let authorized_keys = fs::read_to_string(file_path).map_err(|_| {
        todo!("handle key update failure");
    })?;

    let mut skip_lines = false;
    let mut authorized_keys = authorized_keys
        .lines()
        .filter(|line| {
            let prev = skip_lines;
            skip_lines = match *line {
                "# keyps: START" => true,
                "# keyps: END" => false,
                _ => skip_lines,
            };

            !(prev || skip_lines)
        })
        .collect::<Vec<_>>();

    let mut new_keys = keys
        .iter()
        .filter(|key| !authorized_keys.contains(&key.as_str()))
        .map(|s| s.as_str())
        .collect::<Vec<&str>>();

    authorized_keys.push("# keyps: START");
    authorized_keys.append(&mut new_keys);
    authorized_keys.push("# keyps: END");
    debug!("Writing authorized_keys =>\n{}", authorized_keys.join("\n"));

    fs::write(file_path, authorized_keys.join("\n")).map_err(|_| {
        todo!("handle key update failure");
    })?;

    Ok(())
}
