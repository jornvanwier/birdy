//! src/audio.rs
//! Procedural audio: multi-sample gunshot variations, dynamic spool whine, and thruster roar.

use crate::ship::Thrust;
use crate::ship::weapon::RotaryGun;
use bevy::audio::{PlaybackMode, Volume};
use bevy::prelude::*;
use bevy_rand::global::GlobalRng;
use bevy_rand::prelude::WyRand;
use fundsp::prelude64::*;
use hound::{SampleFormat, WavSpec, WavWriter};
use rand::prelude::*;
use std::io::Cursor;

pub struct ProceduralAudioPlugin;

impl Plugin for ProceduralAudioPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(on_fire_gun_event)
            .add_systems(Startup, setup_audio_assets)
            .add_systems(Update, (update_thruster_audio, update_rotary_spool_audio));
    }
}

// -----------------------------------------------------------------------------
// Resources, Components, and Events
// -----------------------------------------------------------------------------

#[derive(Resource, Clone)]
pub struct SoundBank {
    /// Pool of distinct shot variations to prevent repetitiveness
    pub rotary_shots: Vec<Handle<AudioSource>>,
    pub _rotary_spool_loop: Handle<AudioSource>,
    pub _thruster_loop: Handle<AudioSource>,
}

#[derive(Component)]
pub struct ThrusterAudioSink;

#[derive(Component)]
pub struct RotarySpoolAudioSink;

#[derive(Event, Default)]
pub struct FireGunEvent {
    pub transform: Transform,
}

// -----------------------------------------------------------------------------
// DSP Synthesis Helpers
// -----------------------------------------------------------------------------

const SAMPLE_RATE: u32 = 44100;

fn synth_to_audio_source(mut unit: Box<dyn AudioUnit>, duration_secs: f64) -> AudioSource {
    let spec = WavSpec {
        channels: 1,
        sample_rate: SAMPLE_RATE,
        bits_per_sample: 16,
        sample_format: SampleFormat::Int,
    };

    let mut cursor = Cursor::new(Vec::new());
    {
        let mut writer = WavWriter::new(&mut cursor, spec).expect("Failed to init WavWriter");
        let total_samples = (duration_secs * SAMPLE_RATE as f64) as usize;
        unit.set_sample_rate(SAMPLE_RATE as f64);

        for _ in 0..total_samples {
            let sample = unit.get_mono().clamp(-1.0, 1.0);
            let sample_i16 = (sample * i16::MAX as f32) as i16;
            writer.write_sample(sample_i16).unwrap();
        }
        writer.finalize().unwrap();
    }

    AudioSource {
        bytes: cursor.into_inner().into(),
    }
}

/// 1. Rotary Gun Single Shot Transient (Heavy concussive blast + sharp breech crack)
fn generate_rotary_shot_variant(
    crack_cutoff: f32,
    body_cutoff: f32,
    pitch_start: f64,
    decay_rate: f64,
) -> Box<dyn AudioUnit> {
    // A. Supersonic Muzzle Crack: Ultra-short initial snap (0-8ms)
    let crack_env = envelope(move |t: f64| (-t * 160.0).exp());
    let crack = (white() | constant(crack_cutoff) | constant(2.0)) >> (bandpass() * crack_env * 2.6);

    // B. Concussive Detonation Body: Low-mid shockwave punch
    let body_env = envelope(move |t: f64| (-t * decay_rate).exp());
    let body = (pink() | constant(body_cutoff) | constant(2.2)) >> (bandpass() * body_env * 2.2);

    // C. Heavy Mechanical Punch: Fast pitch drop into sub-bass thump
    let pitch_env = envelope(move |t: f64| pitch_start * (-t * 110.0).exp() + 35.0);
    let punch_amp = envelope(move |t: f64| (-t * 60.0).exp());
    let punch = (pitch_env >> saw()) * punch_amp * 1.4;

    // Saturate through Tanh waveshaper for explosive density
    let mixed = (crack + body + punch) * 1.6;
    Box::new((mixed >> shape(Tanh(1.4)) >> declick()) * 0.9)
}

/// 2. Rotary Gun Motor & Gearbox Whine (Multi-harmonic mechanical gear mesh without noise hiss)
fn generate_rotary_spool_loop() -> Box<dyn AudioUnit> {
    // A. Multi-harmonic planetary gearbox mesh (stacked low-frequency harmonics)
    let gear_1 = (constant(140.0) >> square()) * 0.25;
    let gear_2 = (constant(280.0) >> saw()) * 0.20;
    let gear_3 = (constant(560.0) >> triangle()) * 0.15;

    // Pipe the 3 inputs (signal, cutoff, Q) together into lowpass
    let gear_train = ((gear_1 + gear_2 + gear_3) | constant(900.0) | constant(1.2)) >> lowpass();

    // B. Narrow metallic bearing/housing resonance (high-Q ring, NOT broad noise)
    let bearing_ring = (white() | constant(1350.0) | constant(14.0)) >> (bandpass() * 0.22);

    // C. Heavy electric motor low-end torque
    let motor_core = (constant(70.0) >> saw()) * 0.20;

    let spool_mix = (gear_train + bearing_ring + motor_core) * 1.2;
    Box::new((spool_mix >> shape(Tanh(0.9)) >> declick()) * 0.5)
}

/// Deep combustion & turbine exhaust loop
fn generate_thruster_base_loop() -> Box<dyn AudioUnit> {
    let sub_rumble = (pink() | constant(90.0) | constant(0.9)) >> (lowpass() * 1.8);
    let mid_roar = (pink() | constant(380.0) | constant(1.4)) >> (bandpass() * 1.2);
    let exhaust_hiss = (white() | constant(2200.0) | constant(1.0)) >> (bandpass() * 0.35);

    let engine_mix = (sub_rumble + mid_roar + exhaust_hiss) * 1.3;
    Box::new((engine_mix >> shape(Tanh(1.0)) >> declick()) * 0.8)
}

// -----------------------------------------------------------------------------
// Startup & Asset Setup
// -----------------------------------------------------------------------------

fn setup_audio_assets(mut commands: Commands, mut audio_assets: ResMut<Assets<AudioSource>>) {
    // 5 variations tuned for heavy autocannon impacts
    let shot_params: [(f32, f32, f64, f64); 5] = [
        (3600.0, 420.0, 920.0, 42.0),
        (3800.0, 460.0, 960.0, 40.0),
        (3400.0, 390.0, 890.0, 45.0),
        (4000.0, 490.0, 990.0, 38.0),
        (3500.0, 440.0, 940.0, 41.0),
    ];

    let rotary_shots = shot_params
        .iter()
        .map(|&(c_cut, b_cut, p_start, dec)| {
            audio_assets.add(synth_to_audio_source(
                generate_rotary_shot_variant(c_cut, b_cut, p_start, dec),
                0.08,
            ))
        })
        .collect();

    let rotary_spool_loop =
        audio_assets.add(synth_to_audio_source(generate_rotary_spool_loop(), 0.5));
    let thruster_loop = audio_assets.add(synth_to_audio_source(generate_thruster_base_loop(), 2.0));

    commands.insert_resource(SoundBank {
        rotary_shots,
        _rotary_spool_loop: rotary_spool_loop.clone(),
        _thruster_loop: thruster_loop.clone(),
    });

    // 1. Spool audio player
    commands.spawn((
        AudioPlayer::new(rotary_spool_loop),
        PlaybackSettings {
            mode: PlaybackMode::Loop,
            volume: Volume::SILENT,
            speed: 0.75,
            ..default()
        },
        RotarySpoolAudioSink,
    ));

    // 2. Thruster audio player
    commands.spawn((
        AudioPlayer::new(thruster_loop),
        PlaybackSettings {
            mode: PlaybackMode::Loop,
            volume: Volume::SILENT,
            speed: 0.7,
            ..default()
        },
        ThrusterAudioSink,
    ));
}

// -----------------------------------------------------------------------------
// Runtime Audio Systems
// -----------------------------------------------------------------------------

fn update_thruster_audio(
    thrust_query: Query<&Thrust>,
    mut audio_query: Query<&mut AudioSink, With<ThrusterAudioSink>>,
) {
    let max_throttle = thrust_query
        .iter()
        .map(|t| t.current_throttle)
        .fold(0.0_f32, f32::max)
        .clamp(0.0, 1.0);

    for mut sink in &mut audio_query {
        sink.set_speed(0.85 + max_throttle * 0.45);
        sink.set_volume(Volume::Linear(0.02 + max_throttle * 0.3));
    }
}

/// Dynamically sweeps spool pitch smoothly and ducks volume during sustained firing
fn update_rotary_spool_audio(
    gun_query: Query<&RotaryGun>,
    mut spool_query: Query<&mut AudioSink, With<RotarySpoolAudioSink>>,
) {
    let current_spool = gun_query
        .iter()
        .map(|g| g.current_spool)
        .fold(0.0_f32, f32::max)
        .clamp(0.0, 1.0);

    for mut sink in &mut spool_query {
        if current_spool > 0.01 {
            // Tighter pitch range (0.75x to 1.30x) to eliminate discrete resampling chirps
            sink.set_speed(0.75 + current_spool * 0.55);

            // Smooth linear volume fade (reaches up to 0.45 so it supports, rather than overpowers, gunfire)
            sink.set_volume(Volume::Linear(current_spool * 0.45));
        } else {
            sink.set_volume(Volume::SILENT);
        }
    }
}

/// Plays a shot with randomized sample selection and micro-pitch jitter
fn on_fire_gun_event(
    event: On<FireGunEvent>,
    mut commands: Commands,
    sound_bank: Res<SoundBank>,
    mut rng: Single<&mut WyRand, With<GlobalRng>>,
) {
    if sound_bank.rotary_shots.is_empty() {
        return;
    }

    let shot_handle = sound_bank
        .rotary_shots
        .choose(&mut rng)
        .expect("Sound bank empty")
        .clone();

    // 2. Micro-pitch variation (0.94x .. 1.06x)
    let speed_jitter = rng.random_range(0.94..1.06);

    // 3. Micro-volume variation (0.85x .. 1.0x)
    let volume_jitter = rng.random_range(0.85..1.0);

    commands.spawn((
        AudioPlayer::new(shot_handle),
        PlaybackSettings {
            mode: PlaybackMode::Despawn,
            volume: Volume::Linear(volume_jitter),
            speed: speed_jitter,
            ..default()
        },
        event.transform,
    ));
}
