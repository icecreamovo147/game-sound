//! Real-time CPAL streams and Symphonia decoding.
//!
//! The callbacks in this module deliberately own their audio state. Control
//! changes are passed through fixed-capacity SPSC queues and atomics; callbacks
//! never take a mutex, allocate, decode files, or access the database.
use crate::{
    device,
    mixer::{Levels, MixerSettings},
    sound::Sound,
};
use anyhow::{bail, Context, Result};
use cpal::{
    traits::{DeviceTrait, StreamTrait},
    BufferSize, SampleFormat, SampleRate, Stream, StreamConfig, SupportedBufferSize,
};
use std::{
    cell::UnsafeCell,
    collections::VecDeque,
    fs::File,
    mem::MaybeUninit,
    path::Path,
    sync::{
        atomic::{AtomicBool, AtomicU32, AtomicUsize, Ordering},
        Arc,
    },
};
use symphonia::core::{
    audio::SampleBuffer, codecs::DecoderOptions, formats::FormatOptions, io::MediaSourceStream,
    meta::MetadataOptions, probe::Hint,
};

const MAX_ACTIVE_SOUNDS: usize = 64;
const COMMAND_CAPACITY: usize = 256;
const EVENT_CAPACITY: usize = 512;
const MIC_CAPACITY: usize = 192_000;
const MONITOR_CAPACITY: usize = 192_000;

/// A fixed-capacity lock-free SPSC queue. Each producer/consumer endpoint is
/// created exactly once, which is the ownership model used by CPAL callbacks.
struct Ring<T> {
    slots: Box<[UnsafeCell<MaybeUninit<T>>]>,
    head: AtomicUsize,
    tail: AtomicUsize,
    capacity: usize,
}
unsafe impl<T: Send> Send for Ring<T> {}
unsafe impl<T: Send> Sync for Ring<T> {}
struct Producer<T>(Arc<Ring<T>>);
struct Consumer<T>(Arc<Ring<T>>);
fn ring<T>(capacity: usize) -> (Producer<T>, Consumer<T>) {
    assert!(capacity > 1);
    let slots = (0..capacity)
        .map(|_| UnsafeCell::new(MaybeUninit::uninit()))
        .collect::<Vec<_>>()
        .into_boxed_slice();
    let inner = Arc::new(Ring {
        slots,
        head: AtomicUsize::new(0),
        tail: AtomicUsize::new(0),
        capacity,
    });
    (Producer(inner.clone()), Consumer(inner))
}
impl<T> Producer<T> {
    fn push(&mut self, value: T) -> std::result::Result<(), T> {
        let tail = self.0.tail.load(Ordering::Relaxed);
        let next = (tail + 1) % self.0.capacity;
        if next == self.0.head.load(Ordering::Acquire) {
            return Err(value);
        }
        // SAFETY: only this SPSC producer writes at `tail`; the consumer only
        // reads after `tail` is published with Release ordering.
        unsafe { (*self.0.slots[tail].get()).write(value) };
        self.0.tail.store(next, Ordering::Release);
        Ok(())
    }
}
impl<T> Consumer<T> {
    fn pop(&mut self) -> Option<T> {
        let head = self.0.head.load(Ordering::Relaxed);
        if head == self.0.tail.load(Ordering::Acquire) {
            return None;
        }
        // SAFETY: only this SPSC consumer reads at `head`, after producer's
        // Release publication. It advances head before another read occurs.
        let value = unsafe { (*self.0.slots[head].get()).assume_init_read() };
        self.0
            .head
            .store((head + 1) % self.0.capacity, Ordering::Release);
        Some(value)
    }
}
impl<T> Drop for Ring<T> {
    fn drop(&mut self) {
        let mut head = self.head.load(Ordering::Relaxed);
        let tail = self.tail.load(Ordering::Relaxed);
        while head != tail {
            // SAFETY: Ring is dropped only after its producer/consumer endpoints.
            unsafe { (*self.slots[head].get()).assume_init_drop() };
            head = (head + 1) % self.capacity;
        }
    }
}

#[derive(Clone)]
struct Controls {
    mic_volume: Arc<AtomicU32>,
    sfx_volume: Arc<AtomicU32>,
    monitor_volume: Arc<AtomicU32>,
    mic_muted: Arc<AtomicBool>,
    sfx_muted: Arc<AtomicBool>,
    monitor_muted: Arc<AtomicBool>,
    ducking: Arc<AtomicBool>,
    duck_ratio: Arc<AtomicU32>,
    duck_attack_ms: Arc<AtomicU32>,
    duck_release_ms: Arc<AtomicU32>,
    duck_release_delay_ms: Arc<AtomicU32>,
    level_mic: Arc<AtomicU32>,
    level_output: Arc<AtomicU32>,
    level_monitor: Arc<AtomicU32>,
}
fn atomic_f32(value: f32) -> Arc<AtomicU32> {
    Arc::new(AtomicU32::new(value.to_bits()))
}
fn load_f32(value: &AtomicU32) -> f32 {
    f32::from_bits(value.load(Ordering::Relaxed))
}
impl Controls {
    fn new(settings: MixerSettings) -> Self {
        Self {
            mic_volume: atomic_f32(settings.mic_volume),
            sfx_volume: atomic_f32(settings.sfx_volume),
            monitor_volume: atomic_f32(settings.monitor_volume),
            mic_muted: Arc::new(AtomicBool::new(settings.mic_muted)),
            sfx_muted: Arc::new(AtomicBool::new(settings.sfx_muted)),
            monitor_muted: Arc::new(AtomicBool::new(settings.monitor_muted)),
            ducking: Arc::new(AtomicBool::new(settings.ducking)),
            duck_ratio: atomic_f32(settings.duck_ratio),
            duck_attack_ms: Arc::new(AtomicU32::new(settings.duck_attack_ms)),
            duck_release_ms: Arc::new(AtomicU32::new(settings.duck_release_ms)),
            duck_release_delay_ms: Arc::new(AtomicU32::new(settings.duck_release_delay_ms)),
            level_mic: atomic_f32(0.0),
            level_output: atomic_f32(0.0),
            level_monitor: atomic_f32(0.0),
        }
    }
    fn store(&self, s: MixerSettings) {
        self.mic_volume
            .store(s.mic_volume.clamp(0., 1.).to_bits(), Ordering::Relaxed);
        self.sfx_volume
            .store(s.sfx_volume.clamp(0., 1.).to_bits(), Ordering::Relaxed);
        self.monitor_volume
            .store(s.monitor_volume.clamp(0., 1.).to_bits(), Ordering::Relaxed);
        self.mic_muted.store(s.mic_muted, Ordering::Relaxed);
        self.sfx_muted.store(s.sfx_muted, Ordering::Relaxed);
        self.monitor_muted.store(s.monitor_muted, Ordering::Relaxed);
        self.ducking.store(s.ducking, Ordering::Relaxed);
        self.duck_ratio
            .store(s.duck_ratio.clamp(0., 1.).to_bits(), Ordering::Relaxed);
        self.duck_attack_ms
            .store(s.duck_attack_ms, Ordering::Relaxed);
        self.duck_release_ms
            .store(s.duck_release_ms, Ordering::Relaxed);
        self.duck_release_delay_ms
            .store(s.duck_release_delay_ms, Ordering::Relaxed);
    }
}

#[derive(Clone)]
struct ActiveSound {
    id: i64,
    pcm: Arc<Vec<f32>>,
    cursor: usize,
    volume: f32,
    looped: bool,
    paused: bool,
}
#[derive(Clone)]
pub struct PreparedSound {
    sound: Sound,
    pcm: Arc<Vec<f32>>,
}
#[derive(Debug, Clone, Copy)]
pub struct StreamPreferences {
    pub sample_rate: u32,
    pub channels: u16,
    pub buffer_size: u32,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AudioEvent {
    SoundStarted(i64),
    SoundStopped(i64),
    MicOverrun,
    MonitorOverrun,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StreamFault {
    Mic,
    Output,
    Monitor,
}
#[derive(Debug, Clone, Copy, Default)]
pub struct AudioHealth {
    pub mic_fault: bool,
    pub output_fault: bool,
    pub monitor_fault: bool,
}
enum AudioCommand {
    Play(PreparedSound),
    Stop(i64),
    Pause(i64, bool),
    StopAll,
}

pub struct AudioEngine {
    controls: Controls,
    commands: Option<Producer<AudioCommand>>,
    events: Option<Consumer<AudioEvent>>,
    health: Arc<Health>,
    output_rate: u32,
    output_channels: usize,
    input: Option<Stream>,
    output: Option<Stream>,
    monitor: Option<Stream>,
}
#[derive(Default)]
struct Health {
    mic: AtomicBool,
    output: AtomicBool,
    monitor: AtomicBool,
}
impl Default for AudioEngine {
    fn default() -> Self {
        Self {
            controls: Controls::new(MixerSettings::default()),
            commands: None,
            events: None,
            health: Arc::new(Health::default()),
            output_rate: 48_000,
            output_channels: 2,
            input: None,
            output: None,
            monitor: None,
        }
    }
}

fn apply_preference(
    mut config: StreamConfig,
    supported: &SupportedBufferSize,
    p: StreamPreferences,
) -> StreamConfig {
    if let SupportedBufferSize::Range { min, max } = supported {
        if p.buffer_size >= *min && p.buffer_size <= *max {
            config.buffer_size = BufferSize::Fixed(p.buffer_size);
        }
    }
    config
}
fn preferred_output_config(
    device: &cpal::Device,
    p: StreamPreferences,
) -> Result<(StreamConfig, SampleFormat)> {
    let fallback = device
        .default_output_config()
        .context("output has no default configuration")?;
    let selected = device.supported_output_configs()?.find_map(|r| {
        (r.channels() == p.channels)
            .then(|| r.try_with_sample_rate(SampleRate(p.sample_rate)))
            .flatten()
    });
    Ok(selected
        .map(|c| {
            (
                apply_preference(c.config(), c.buffer_size(), p),
                c.sample_format(),
            )
        })
        .unwrap_or((fallback.config(), fallback.sample_format())))
}
fn preferred_input_config(
    device: &cpal::Device,
    p: StreamPreferences,
) -> Result<(StreamConfig, SampleFormat)> {
    let fallback = device
        .default_input_config()
        .context("input has no default configuration")?;
    let selected = device.supported_input_configs()?.find_map(|r| {
        (r.channels() == p.channels)
            .then(|| r.try_with_sample_rate(SampleRate(p.sample_rate)))
            .flatten()
    });
    Ok(selected
        .map(|c| {
            (
                apply_preference(c.config(), c.buffer_size(), p),
                c.sample_format(),
            )
        })
        .unwrap_or((fallback.config(), fallback.sample_format())))
}

impl AudioEngine {
    pub fn levels(&self) -> Levels {
        Levels {
            mic: load_f32(&self.controls.level_mic),
            output: load_f32(&self.controls.level_output),
            monitor: load_f32(&self.controls.level_monitor),
        }
    }
    pub fn output_format(&self) -> (u32, usize) {
        (self.output_rate, self.output_channels)
    }
    pub fn set_settings(&self, settings: MixerSettings) {
        self.controls.store(settings);
    }
    pub fn drain_events(&mut self) -> Vec<AudioEvent> {
        let mut events = Vec::new();
        if let Some(queue) = &mut self.events {
            while let Some(e) = queue.pop() {
                events.push(e);
            }
        }
        events
    }
    pub fn take_health(&self) -> AudioHealth {
        AudioHealth {
            mic_fault: self.health.mic.swap(false, Ordering::Relaxed),
            output_fault: self.health.output.swap(false, Ordering::Relaxed),
            monitor_fault: self.health.monitor.swap(false, Ordering::Relaxed),
        }
    }
    fn command(&mut self, command: AudioCommand) -> Result<()> {
        self.commands
            .as_mut()
            .context("audio engine is not running; choose devices and start it first")?
            .push(command)
            .map_err(|_| anyhow::anyhow!("audio control queue is full"))
    }
    pub fn stop_all(&mut self) -> Result<()> {
        self.command(AudioCommand::StopAll)
    }
    pub fn stop_sound(&mut self, id: i64) -> Result<()> {
        self.command(AudioCommand::Stop(id))
    }
    pub fn set_paused(&mut self, id: i64, paused: bool) -> Result<()> {
        self.command(AudioCommand::Pause(id, paused))
    }
    pub fn prepare(
        sound: Sound,
        output_rate: u32,
        output_channels: usize,
    ) -> Result<PreparedSound> {
        if !sound.is_available() {
            bail!("audio file is missing: {}", sound.file_path);
        }
        let decoded = decode_file_with_format(Path::new(&sound.file_path))?;
        let pcm = Arc::new(convert_pcm(
            decoded.samples,
            decoded.sample_rate,
            decoded.channels,
            output_rate,
            output_channels,
        ));
        if pcm.is_empty() {
            bail!("audio contains no PCM samples");
        }
        Ok(PreparedSound { sound, pcm })
    }
    pub fn play_prepared(&mut self, prepared: PreparedSound) -> Result<()> {
        self.command(AudioCommand::Play(prepared))
    }
    pub fn start(
        &mut self,
        mic: Option<&str>,
        output: &str,
        monitor: Option<&str>,
        monitor_sfx_only: bool,
        preferences: StreamPreferences,
    ) -> Result<()> {
        self.shutdown();
        let output_device = device::find_output(output)?;
        let (output_config, output_format) = preferred_output_config(&output_device, preferences)?;
        let output_rate = output_config.sample_rate.0;
        let output_channels = output_config.channels as usize;
        let (mic_producer, mic_consumer) = ring(MIC_CAPACITY);
        let (commands, command_consumer) = ring(COMMAND_CAPACITY);
        let (event_producer, event_consumer) = ring(EVENT_CAPACITY);
        let (mix_producer, mix_consumer) = ring(MONITOR_CAPACITY);
        let (sfx_producer, sfx_consumer) = ring(MONITOR_CAPACITY);
        let output_stream = build_output(
            output_device,
            output_config,
            output_format,
            OutputCallback {
                mic: mic_consumer,
                commands: command_consumer,
                events: event_producer,
                controls: self.controls.clone(),
                mix_monitor: if monitor.is_some() {
                    Some(mix_producer)
                } else {
                    None
                },
                sfx_monitor: if monitor.is_some() {
                    Some(sfx_producer)
                } else {
                    None
                },
                output_rate,
                output_channels,
            },
            self.health.clone(),
        )?;
        let input_stream = match mic {
            Some(id) => Some(build_input(
                device::find_input(id)?,
                preferences,
                output_rate,
                output_channels,
                mic_producer,
                self.health.clone(),
            )?),
            None => None,
        };
        let monitor_stream = match monitor {
            Some(id) => {
                let d = device::find_output(id)?;
                let (cfg, fmt) = preferred_output_config(&d, preferences)?;
                Some(build_monitor(
                    d,
                    cfg,
                    fmt,
                    if monitor_sfx_only {
                        sfx_consumer
                    } else {
                        mix_consumer
                    },
                    self.health.clone(),
                    self.controls.clone(),
                )?)
            }
            None => None,
        };
        // No stream plays until every required stream is built successfully.
        if let Err(error) = output_stream
            .play()
            .and_then(|_| input_stream.as_ref().map(StreamTrait::play).transpose())
            .and_then(|_| monitor_stream.as_ref().map(StreamTrait::play).transpose())
        {
            drop(monitor_stream);
            drop(input_stream);
            drop(output_stream);
            return Err(error.into());
        }
        self.output_rate = output_rate;
        self.output_channels = output_channels;
        self.commands = Some(commands);
        self.events = Some(event_consumer);
        self.output = Some(output_stream);
        self.input = input_stream;
        self.monitor = monitor_stream;
        Ok(())
    }
    pub fn shutdown(&mut self) {
        self.input.take();
        self.output.take();
        self.monitor.take();
        self.commands.take();
        self.events.take();
    }
}

struct InputConverter {
    source_rate: u32,
    target_rate: u32,
    source_channels: usize,
    target_channels: usize,
    frame: u64,
    next_position: f64,
    previous: Vec<f32>,
    current: Vec<f32>,
    mixed: Vec<f32>,
    initialized: bool,
}
impl InputConverter {
    fn new(
        source_rate: u32,
        target_rate: u32,
        source_channels: usize,
        target_channels: usize,
    ) -> Self {
        Self {
            source_rate,
            target_rate,
            source_channels,
            target_channels,
            frame: 0,
            next_position: 0.,
            previous: vec![0.; target_channels],
            current: vec![0.; source_channels],
            mixed: vec![0.; target_channels],
            initialized: false,
        }
    }
    fn process<I: Iterator<Item = f32>>(
        &mut self,
        mut data: I,
        producer: &mut Producer<f32>,
        health: &Health,
    ) {
        if self.source_channels == 0 {
            return;
        }
        loop {
            for channel in 0..self.source_channels {
                let Some(sample) = data.next() else {
                    return;
                };
                self.current[channel] = sample;
            }
            Self::normalized_frame_for(
                self.source_channels,
                self.target_channels,
                &self.current,
                &mut self.mixed,
            );
            if !self.initialized {
                self.previous.copy_from_slice(&self.mixed);
                self.initialized = true;
                self.frame = 0;
                continue;
            }
            let current = self.frame + 1;
            while self.next_position <= current as f64 {
                let fraction = (self.next_position - self.frame as f64).clamp(0., 1.) as f32;
                for channel in 0..self.target_channels {
                    if producer
                        .push(
                            self.previous[channel]
                                + (self.mixed[channel] - self.previous[channel]) * fraction,
                        )
                        .is_err()
                    {
                        health.mic.store(true, Ordering::Relaxed);
                        return;
                    }
                }
                self.next_position += self.source_rate as f64 / self.target_rate.max(1) as f64;
            }
            self.previous.copy_from_slice(&self.mixed);
            self.frame = current;
        }
    }
    fn normalized_frame_for(
        source_channels: usize,
        target_channels: usize,
        samples: &[f32],
        output: &mut [f32],
    ) {
        if target_channels == 1 {
            output[0] = samples.iter().copied().sum::<f32>() / source_channels.max(1) as f32;
            return;
        }
        for (channel, destination) in output.iter_mut().enumerate().take(target_channels) {
            let start = channel * source_channels / target_channels;
            let end = ((channel + 1) * source_channels / target_channels)
                .max(start + 1)
                .min(source_channels);
            *destination = samples[start..end].iter().copied().sum::<f32>() / (end - start) as f32;
        }
    }
}

fn build_input(
    device: cpal::Device,
    preferences: StreamPreferences,
    output_rate: u32,
    output_channels: usize,
    producer: Producer<f32>,
    health: Arc<Health>,
) -> Result<Stream> {
    let (cfg, format) = preferred_input_config(&device, preferences)?;
    let converter = InputConverter::new(
        cfg.sample_rate.0,
        output_rate,
        cfg.channels as usize,
        output_channels,
    );
    macro_rules! input {
        ($type:ty, $convert:expr) => {{
            let mut converter = converter;
            let mut producer = producer;
            let callback_health = health.clone();
            let error_health = health.clone();
            device.build_input_stream(
                &cfg,
                move |data: &[$type], _| {
                    converter.process(
                        data.iter().copied().map($convert),
                        &mut producer,
                        &callback_health,
                    )
                },
                move |_| error_health.mic.store(true, Ordering::Relaxed),
                None,
            )?
        }};
    }
    Ok(match format {
        SampleFormat::F32 => input!(f32, |x: f32| x),
        SampleFormat::I16 => input!(i16, |x: i16| x as f32 / i16::MAX as f32),
        SampleFormat::U16 => input!(u16, |x: u16| (x as f32 / u16::MAX as f32) * 2. - 1.),
        other => bail!("unsupported input sample format: {other:?}"),
    })
}

struct OutputCallback {
    mic: Consumer<f32>,
    commands: Consumer<AudioCommand>,
    events: Producer<AudioEvent>,
    controls: Controls,
    mix_monitor: Option<Producer<f32>>,
    sfx_monitor: Option<Producer<f32>>,
    output_rate: u32,
    output_channels: usize,
}
struct OutputState {
    active: Vec<ActiveSound>,
    queued: VecDeque<ActiveSound>,
    duck_gain: f32,
    release_wait: u32,
}
impl OutputState {
    fn new() -> Self {
        Self {
            active: Vec::with_capacity(MAX_ACTIVE_SOUNDS),
            queued: VecDeque::with_capacity(MAX_ACTIVE_SOUNDS),
            duck_gain: 1.,
            release_wait: 0,
        }
    }
    fn consume_commands(&mut self, callback: &mut OutputCallback) {
        while let Some(command) = callback.commands.pop() {
            match command {
                AudioCommand::Play(prepared) => {
                    let sound = prepared.sound;
                    if matches!(sound.playback_mode, crate::sound::PlaybackMode::Interrupt) {
                        for active in self.active.drain(..) {
                            let _ = callback.events.push(AudioEvent::SoundStopped(active.id));
                        }
                    }
                    if matches!(sound.playback_mode, crate::sound::PlaybackMode::Exclusive) {
                        let mut i = 0;
                        while i < self.active.len() {
                            if self.active[i].id == sound.id {
                                let stopped = self.active.remove(i);
                                let _ = callback.events.push(AudioEvent::SoundStopped(stopped.id));
                            } else {
                                i += 1;
                            }
                        }
                    }
                    let instance = ActiveSound {
                        id: sound.id,
                        pcm: prepared.pcm,
                        cursor: 0,
                        volume: sound.volume,
                        looped: sound.loop_enabled,
                        paused: false,
                    };
                    if matches!(sound.playback_mode, crate::sound::PlaybackMode::Queue)
                        && !self.active.is_empty()
                    {
                        if self.queued.len() < MAX_ACTIVE_SOUNDS {
                            self.queued.push_back(instance);
                        }
                    } else if self.active.len() < MAX_ACTIVE_SOUNDS {
                        self.active.push(instance);
                        let _ = callback.events.push(AudioEvent::SoundStarted(sound.id));
                    }
                }
                AudioCommand::Stop(id) => {
                    self.active.retain(|s| {
                        if s.id == id {
                            let _ = callback.events.push(AudioEvent::SoundStopped(id));
                            false
                        } else {
                            true
                        }
                    });
                    self.queued.retain(|s| s.id != id);
                }
                AudioCommand::Pause(id, pause) => {
                    for s in &mut self.active {
                        if s.id == id {
                            s.paused = pause;
                        }
                    }
                }
                AudioCommand::StopAll => {
                    for s in self.active.drain(..) {
                        let _ = callback.events.push(AudioEvent::SoundStopped(s.id));
                    }
                    self.queued.clear();
                }
            }
        }
    }
    fn render(&mut self, callback: &mut OutputCallback, target: &mut [f32]) {
        self.consume_commands(callback);
        let frames = target.len() / callback.output_channels.max(1);
        let mut mic_peak = 0.0f32;
        let mut output_peak = 0.0f32;
        for frame in 0..frames {
            let active = self.active.iter().any(|s| !s.paused);
            let target_gain = if active && callback.controls.ducking.load(Ordering::Relaxed) {
                load_f32(&callback.controls.duck_ratio)
            } else {
                1.
            };
            if active {
                self.release_wait = callback
                    .controls
                    .duck_release_delay_ms
                    .load(Ordering::Relaxed);
            } else if self.release_wait > 0 {
                self.release_wait -= 1;
            }
            let millis = if target_gain < self.duck_gain {
                callback.controls.duck_attack_ms.load(Ordering::Relaxed)
            } else if self.release_wait == 0 {
                callback.controls.duck_release_ms.load(Ordering::Relaxed)
            } else {
                u32::MAX
            };
            if millis != u32::MAX {
                let step =
                    1. / ((callback.output_rate as f32 * millis.max(1) as f32) / 1000.).max(1.);
                self.duck_gain += (target_gain - self.duck_gain).clamp(-step, step);
            }
            let mic_gain = if callback.controls.mic_muted.load(Ordering::Relaxed) {
                0.
            } else {
                load_f32(&callback.controls.mic_volume) * self.duck_gain
            };
            let sfx_gain = if callback.controls.sfx_muted.load(Ordering::Relaxed) {
                0.
            } else {
                load_f32(&callback.controls.sfx_volume)
            };
            for channel in 0..callback.output_channels {
                let mic = callback.mic.pop().unwrap_or(0.);
                let mut sfx = 0.;
                for sound in &mut self.active {
                    if !sound.paused && sound.cursor < sound.pcm.len() {
                        sfx += sound.pcm[sound.cursor] * sound.volume;
                        sound.cursor += 1;
                    } else if !sound.paused && sound.looped && !sound.pcm.is_empty() {
                        sound.cursor = 1;
                        sfx += sound.pcm[0] * sound.volume;
                    }
                }
                let mixed = soft_limit(mic * mic_gain + sfx * sfx_gain);
                target[frame * callback.output_channels + channel] = mixed;
                mic_peak = mic_peak.max(mic.abs());
                output_peak = output_peak.max(mixed.abs());
                if let Some(ring) = &mut callback.mix_monitor {
                    if ring.push(mixed).is_err() {
                        let _ = callback.events.push(AudioEvent::MonitorOverrun);
                    }
                }
                if let Some(ring) = &mut callback.sfx_monitor {
                    if ring
                        .push(sfx * sfx_gain * load_f32(&callback.controls.monitor_volume))
                        .is_err()
                    {
                        let _ = callback.events.push(AudioEvent::MonitorOverrun);
                    }
                }
            }
            let mut i = 0;
            while i < self.active.len() {
                if !self.active[i].looped && self.active[i].cursor >= self.active[i].pcm.len() {
                    let stopped = self.active.remove(i);
                    let _ = callback.events.push(AudioEvent::SoundStopped(stopped.id));
                } else {
                    i += 1;
                }
            }
            if self.active.is_empty() {
                if let Some(next) = self.queued.pop_front() {
                    let id = next.id;
                    self.active.push(next);
                    let _ = callback.events.push(AudioEvent::SoundStarted(id));
                }
            }
        }
        callback
            .controls
            .level_mic
            .store(mic_peak.to_bits(), Ordering::Relaxed);
        callback
            .controls
            .level_output
            .store(output_peak.to_bits(), Ordering::Relaxed);
        callback.controls.level_monitor.store(
            (output_peak * load_f32(&callback.controls.monitor_volume)).to_bits(),
            Ordering::Relaxed,
        );
    }
}
fn build_output(
    device: cpal::Device,
    cfg: StreamConfig,
    format: SampleFormat,
    callback: OutputCallback,
    health: Arc<Health>,
) -> Result<Stream> {
    macro_rules! output {
        ($type:ty, $convert:expr) => {{
            let mut callback = callback;
            let mut state = OutputState::new();
            let error_health = health.clone();
            device.build_output_stream(
                &cfg,
                move |data: &mut [$type], _| {
                    let mut scratch = [0f32; 4096];
                    if data.len() > scratch.len() {
                        data.fill($convert(0.0));
                        return;
                    }
                    state.render(&mut callback, &mut scratch[..data.len()]);
                    for (out, sample) in data.iter_mut().zip(scratch) {
                        *out = $convert(sample);
                    }
                },
                move |_| error_health.output.store(true, Ordering::Relaxed),
                None,
            )?
        }};
    }
    Ok(match format {
        SampleFormat::F32 => output!(f32, |x: f32| x),
        SampleFormat::I16 => output!(i16, |x: f32| (x.clamp(-1., 1.) * i16::MAX as f32) as i16),
        SampleFormat::U16 => output!(
            u16,
            |x: f32| ((x.clamp(-1., 1.) + 1.) * 0.5 * u16::MAX as f32) as u16
        ),
        other => bail!("unsupported output sample format: {other:?}"),
    })
}
fn build_monitor(
    device: cpal::Device,
    cfg: StreamConfig,
    format: SampleFormat,
    consumer: Consumer<f32>,
    health: Arc<Health>,
    controls: Controls,
) -> Result<Stream> {
    macro_rules! monitor {
        ($type:ty, $convert:expr) => {{
            let mut consumer = consumer;
            let error_health = health.clone();
            let controls = controls.clone();
            device.build_output_stream(
                &cfg,
                move |data: &mut [$type], _| {
                    for sample in data {
                        let value = consumer.pop().unwrap_or(0.0);
                        *sample = $convert(if controls.monitor_muted.load(Ordering::Relaxed) {
                            0.0
                        } else {
                            value
                        });
                    }
                },
                move |_| error_health.monitor.store(true, Ordering::Relaxed),
                None,
            )?
        }};
    }
    Ok(match format {
        SampleFormat::F32 => monitor!(f32, |x: f32| x),
        SampleFormat::I16 => monitor!(i16, |x: f32| (x.clamp(-1., 1.) * i16::MAX as f32) as i16),
        SampleFormat::U16 => monitor!(u16, |x: f32| ((x.clamp(-1., 1.) + 1.)
            * 0.5
            * u16::MAX as f32) as u16),
        other => bail!("unsupported monitor sample format: {other:?}"),
    })
}

pub fn soft_limit(sample: f32) -> f32 {
    sample.tanh()
}
pub struct DecodedPcm {
    pub samples: Vec<f32>,
    pub sample_rate: u32,
    pub channels: usize,
}
pub fn decode_file(path: &Path) -> Result<Vec<f32>> {
    Ok(decode_file_with_format(path)?.samples)
}
pub fn probe_file(path: &Path) -> Result<(u32, usize, u64)> {
    let d = decode_file_with_format(path)?;
    Ok((d.sample_rate, d.channels, d.samples.len() as u64))
}
fn decode_file_with_format(path: &Path) -> Result<DecodedPcm> {
    let file = File::open(path).with_context(|| format!("cannot open {}", path.display()))?;
    let mss = MediaSourceStream::new(Box::new(file), Default::default());
    let mut hint = Hint::new();
    if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
        hint.with_extension(ext);
    }
    let probed = symphonia::default::get_probe().format(
        &hint,
        mss,
        &FormatOptions::default(),
        &MetadataOptions::default(),
    )?;
    let mut format = probed.format;
    let track = format.default_track().context("no default audio track")?;
    let track_id = track.id;
    let mut decoder =
        symphonia::default::get_codecs().make(&track.codec_params, &DecoderOptions::default())?;
    let sample_rate = track.codec_params.sample_rate.unwrap_or(48_000);
    let channels = track.codec_params.channels.map(|c| c.count()).unwrap_or(2);
    let mut samples = Vec::new();
    while let Ok(packet) = format.next_packet() {
        if packet.track_id() != track_id {
            continue;
        }
        match decoder.decode(&packet) {
            Ok(buffer) => {
                let mut out = SampleBuffer::<f32>::new(buffer.capacity() as u64, *buffer.spec());
                out.copy_interleaved_ref(buffer);
                samples.extend_from_slice(out.samples());
            }
            Err(symphonia::core::errors::Error::DecodeError(_)) => continue,
            Err(error) => return Err(error.into()),
        }
    }
    Ok(DecodedPcm {
        samples,
        sample_rate,
        channels,
    })
}
fn convert_pcm(
    input: Vec<f32>,
    source_rate: u32,
    source_channels: usize,
    target_rate: u32,
    target_channels: usize,
) -> Vec<f32> {
    if input.is_empty() || source_channels == 0 || target_channels == 0 {
        return Vec::new();
    }
    let source_frames = input.len() / source_channels;
    let target_frames =
        ((source_frames as f64 * target_rate as f64 / source_rate.max(1) as f64).ceil()) as usize;
    let mut output = Vec::with_capacity(target_frames * target_channels);
    for frame in 0..target_frames {
        let position = frame as f64 * source_rate as f64 / target_rate.max(1) as f64;
        let lower = position.floor() as usize;
        let upper = (lower + 1).min(source_frames.saturating_sub(1));
        let fraction = (position - lower as f64) as f32;
        for channel in 0..target_channels {
            let mix = |index: usize| {
                let start = channel * source_channels / target_channels;
                let end = ((channel + 1) * source_channels / target_channels)
                    .max(start + 1)
                    .min(source_channels);
                input[index * source_channels + start..index * source_channels + end]
                    .iter()
                    .copied()
                    .sum::<f32>()
                    / (end - start) as f32
            };
            output.push(mix(lower) + (mix(upper) - mix(lower)) * fraction);
        }
    }
    output
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn ring_moves_without_locks() {
        let (mut p, mut c) = ring(4);
        assert!(p.push(1).is_ok());
        assert_eq!(c.pop(), Some(1));
    }
    #[test]
    fn downmix_uses_both_stereo_channels() {
        let output = convert_pcm(vec![0., 1.], 48_000, 2, 48_000, 1);
        assert_eq!(output, vec![0.5]);
    }
    #[test]
    fn streaming_resampler_preserves_fractional_progress_across_callbacks() {
        let (mut producer, mut consumer) = ring(64);
        let health = Health::default();
        let mut converter = InputConverter::new(3, 4, 1, 1);
        converter.process([0.0, 0.5].into_iter(), &mut producer, &health);
        converter.process([1.0, 0.5].into_iter(), &mut producer, &health);
        let samples = std::iter::from_fn(|| consumer.pop()).collect::<Vec<_>>();
        assert!(samples.len() >= 4);
        assert!(samples
            .windows(2)
            .all(|pair| (pair[1] - pair[0]).abs() <= 0.75));
    }
    #[test]
    fn monitor_queue_consumes_each_frame_once_without_cycling() {
        let (mut producer, mut consumer) = ring(8);
        for sample in [0.1, 0.2, 0.3] {
            producer.push(sample).unwrap();
        }
        assert_eq!(consumer.pop(), Some(0.1));
        assert_eq!(consumer.pop(), Some(0.2));
        assert_eq!(consumer.pop(), Some(0.3));
        assert_eq!(consumer.pop(), None);
    }
    #[test]
    fn failed_device_start_leaves_no_streams_running() {
        let mut engine = AudioEngine::default();
        assert!(engine
            .start(
                None,
                "definitely unavailable GameSound output",
                None,
                true,
                StreamPreferences {
                    sample_rate: 48_000,
                    channels: 2,
                    buffer_size: 512
                }
            )
            .is_err());
        assert!(engine.output.is_none());
        assert!(engine.input.is_none());
        assert!(engine.monitor.is_none());
    }
    #[test]
    fn duck_envelope_is_not_a_step() {
        let mut gain = 1.0f32;
        let target = 0.4f32;
        let step = 1.0f32 / 100.0;
        gain += (target - gain).clamp(-step, step);
        assert!(gain < 1. && gain > target);
    }
}
