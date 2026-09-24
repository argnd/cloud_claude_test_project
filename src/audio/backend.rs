//! rodio output. The device is opened on a background thread (opening can be
//! slow on some systems), which then just keeps the stream alive. The engine
//! itself runs inside a rodio `Source` on the audio callback thread and
//! receives commands through a channel, rendering in small blocks.

use std::num::NonZero;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{Receiver, Sender, channel};
use std::time::{Duration, Instant};

use super::engine::{Cmd, Engine};

/// Frames rendered per refill of the source buffer.
const FRAMES: usize = 512;
/// Sound effects older than this when the engine gets to them are dropped
/// (e.g. requested while the device was still opening).
const STALE: Duration = Duration::from_millis(250);

struct Msg {
    cmd: Cmd,
    at: Instant,
}

pub struct Backend {
    tx: Sender<Msg>,
    alive: Arc<AtomicBool>,
    /// Dropping this ends the device thread, which closes the stream.
    _stop: Sender<()>,
}

impl Backend {
    /// Starts the device thread. Returns immediately; if no device can be
    /// opened the backend marks itself dead and all sends become no-ops.
    pub fn start() -> Option<Backend> {
        let (tx, rx) = channel::<Msg>();
        let (stop_tx, stop_rx) = channel::<()>();
        let alive = Arc::new(AtomicBool::new(true));
        let flag = alive.clone();
        let spawned = std::thread::Builder::new()
            .name("emberdeep-audio".into())
            .spawn(move || {
                let opened = catch_unwind(rodio::DeviceSinkBuilder::open_default_sink);
                let mut sink = match opened {
                    Ok(Ok(sink)) => sink,
                    _ => {
                        flag.store(false, Ordering::Relaxed);
                        return;
                    }
                };
                sink.log_on_drop(false);
                let rate = sink.config().sample_rate();
                sink.mixer().add(EngineSource::new(rate, rx, flag));
                // Keep the stream open until `Audio` is dropped.
                let _ = stop_rx.recv();
                drop(sink);
            });
        match spawned {
            Ok(_) => Some(Backend {
                tx,
                alive,
                _stop: stop_tx,
            }),
            Err(_) => None,
        }
    }

    pub fn alive(&self) -> bool {
        self.alive.load(Ordering::Relaxed)
    }

    pub fn send(&self, cmd: Cmd) {
        if self
            .tx
            .send(Msg {
                cmd,
                at: Instant::now(),
            })
            .is_err()
        {
            self.alive.store(false, Ordering::Relaxed);
        }
    }
}

/// The engine as an endless stereo rodio source.
struct EngineSource {
    engine: Engine,
    rx: Receiver<Msg>,
    buf: Vec<f32>,
    pos: usize,
    rate: NonZero<u32>,
    failed: bool,
    alive: Arc<AtomicBool>,
}

impl EngineSource {
    fn new(rate: NonZero<u32>, rx: Receiver<Msg>, alive: Arc<AtomicBool>) -> EngineSource {
        EngineSource {
            engine: Engine::new(rate.get()),
            rx,
            buf: vec![0.0; FRAMES * 2],
            pos: FRAMES * 2,
            rate,
            failed: false,
            alive,
        }
    }

    fn refill(&mut self) {
        self.pos = 0;
        if self.failed {
            self.buf.fill(0.0);
            return;
        }
        let engine = &mut self.engine;
        let rx = &self.rx;
        let buf = &mut self.buf;
        let result = catch_unwind(AssertUnwindSafe(|| {
            while let Ok(msg) = rx.try_recv() {
                if matches!(msg.cmd, Cmd::Sfx(_)) && msg.at.elapsed() > STALE {
                    continue;
                }
                drop(engine.command(msg.cmd));
            }
            engine.render_interleaved(buf);
        }));
        if result.is_err() {
            // Never take the game down with the audio: go quiet instead.
            self.failed = true;
            self.alive.store(false, Ordering::Relaxed);
            self.buf.fill(0.0);
        }
    }
}

impl Iterator for EngineSource {
    type Item = rodio::Sample;

    #[inline]
    fn next(&mut self) -> Option<rodio::Sample> {
        if self.pos >= self.buf.len() {
            self.refill();
        }
        let s = self.buf[self.pos];
        self.pos += 1;
        Some(s as rodio::Sample)
    }
}

impl rodio::Source for EngineSource {
    fn current_span_len(&self) -> Option<usize> {
        None
    }

    fn channels(&self) -> rodio::ChannelCount {
        NonZero::new(2).unwrap()
    }

    fn sample_rate(&self) -> rodio::SampleRate {
        self.rate
    }

    fn total_duration(&self) -> Option<Duration> {
        None
    }
}
