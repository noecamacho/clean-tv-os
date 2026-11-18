//! Volume, OLED light and radios, read and written off the UI thread.
//!
//! A Luna call can take seconds (the first `getVolume` after launch took 6.5 s on the C1), and
//! the Control Center polls the levels while it is open: done on the loop, that froze drawing
//! and the remote. A worker thread with its own bus connection does the calls; the loop sends
//! jobs and picks up the answers without waiting.

use crate::bridge::Bridge;
use std::path::Path;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::time::{Duration, Instant};

pub enum Job {
    Read,
    Radios,
    SetVolume(u8),
    SetBacklight(u8),
}

pub enum Done {
    Levels { volume: u8, backlight: u8 },
    Radios { wifi: bool, bluetooth: bool },
}

pub struct Levels {
    jobs: Sender<Job>,
    done: Receiver<Done>,
    /// A read is in flight: polling again before it answers would only queue duplicates.
    reading: bool,
}

/// What one batch of queued jobs asks for; a slider sends many sets, only the last matters.
#[derive(Default)]
struct Batch {
    read: bool,
    radios: bool,
    volume: Option<u8>,
    backlight: Option<u8>,
}

impl Batch {
    fn add(&mut self, job: Job) {
        match job {
            Job::Read => self.read = true,
            Job::Radios => self.radios = true,
            Job::SetVolume(v) => self.volume = Some(v),
            Job::SetBacklight(v) => self.backlight = Some(v),
        }
    }
}

impl Levels {
    pub fn start(data_dir: &Path) -> Levels {
        let (jobs, inbox) = channel::<Job>();
        let (outbox, done) = channel::<Done>();
        let data_dir = data_dir.to_path_buf();
        let spawned = std::thread::Builder::new()
            .name("levels".into())
            .spawn(move || {
                let bridge = Bridge::new(&data_dir);
                while let Ok(first) = inbox.recv() {
                    let mut batch = Batch::default();
                    batch.add(first);
                    while let Ok(next) = inbox.try_recv() {
                        batch.add(next);
                    }
                    if let Some(v) = batch.volume {
                        batch.read |= !bridge.set_volume(v);
                    }
                    if let Some(v) = batch.backlight {
                        batch.read |= !bridge.set_backlight(v);
                    }
                    if batch.read {
                        let started = Instant::now();
                        let (volume, backlight) = (bridge.volume(), bridge.backlight());
                        if started.elapsed() >= Duration::from_millis(100) {
                            log::warn!(
                                "reading the levels took {} ms",
                                started.elapsed().as_millis()
                            );
                        }
                        // Always answer a read, so the loop may ask again; 0/0 is never shown
                        // because a failed read keeps the last levels (see the caller).
                        let levels = Done::Levels {
                            volume: volume.unwrap_or(u8::MAX),
                            backlight: backlight.unwrap_or(u8::MAX),
                        };
                        if outbox.send(levels).is_err() {
                            break;
                        }
                    }
                    if batch.radios {
                        if let Some((wifi, bluetooth)) = bridge.radios() {
                            if outbox.send(Done::Radios { wifi, bluetooth }).is_err() {
                                break;
                            }
                        }
                    }
                }
            });
        if let Err(e) = spawned {
            log::error!("cannot start the levels thread: {e}");
        }
        Levels {
            jobs,
            done,
            reading: false,
        }
    }

    pub fn send(&self, job: Job) {
        let _ = self.jobs.send(job);
    }

    pub fn request_read(&mut self) {
        if !self.reading {
            self.reading = self.jobs.send(Job::Read).is_ok();
        }
    }

    /// The next answer, if one arrived. A level of `u8::MAX` means that read failed.
    pub fn try_recv(&mut self) -> Option<Done> {
        let done = self.done.try_recv().ok();
        if matches!(done, Some(Done::Levels { .. })) {
            self.reading = false;
        }
        done
    }
}
