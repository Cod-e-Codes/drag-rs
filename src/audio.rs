use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Device, Stream, StreamConfig};
use std::sync::{Arc, Mutex};

pub struct AudioEngine {
    _stream: Stream,
    state: Arc<Mutex<AudioState>>,
}

#[derive(Clone)]
struct AudioState {
    engine_frequency: f32,
    engine_amplitude: f32,
    beep_timer: f32,
    beep_frequency: f32,
    beep_active: bool,
}

impl AudioEngine {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let host = cpal::default_host();
        let device = host
            .default_output_device()
            .ok_or("No output device available")?;

        let config = device.default_output_config()?;

        let state = Arc::new(Mutex::new(AudioState {
            engine_frequency: 0.0,
            engine_amplitude: 0.0,
            beep_timer: 0.0,
            beep_frequency: 0.0,
            beep_active: false,
        }));

        let stream = Self::build_stream(&device, &config.into(), state.clone())?;
        stream.play()?;

        Ok(Self {
            _stream: stream,
            state,
        })
    }

    fn build_stream(
        device: &Device,
        config: &StreamConfig,
        state: Arc<Mutex<AudioState>>,
    ) -> Result<Stream, Box<dyn std::error::Error>> {
        let sample_rate = config.sample_rate.0 as f32;
        let channels = config.channels as usize;
        let mut sample_clock = 0f32;

        let stream = device.build_output_stream(
            config,
            move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                let state = state.lock().unwrap();

                for frame in data.chunks_mut(channels) {
                    let time = sample_clock / sample_rate;
                    sample_clock += 1.0;

                    // Generate engine sound (sawtooth wave with harmonics for realism)
                    let engine_sample = if state.engine_amplitude > 0.0 {
                        let fundamental = Self::sawtooth(time, state.engine_frequency);
                        let harmonic2 = Self::sawtooth(time, state.engine_frequency * 2.0) * 0.3;
                        let harmonic3 = Self::sawtooth(time, state.engine_frequency * 3.0) * 0.15;

                        // Add some noise for realism
                        let noise = (sample_clock % 100.0) / 100.0 - 0.5;

                        (fundamental + harmonic2 + harmonic3 + noise * 0.1) * state.engine_amplitude
                    } else {
                        0.0
                    };

                    // Generate beep sound (sine wave)
                    let beep_sample = if state.beep_active {
                        // Use envelope to prevent clicking
                        let envelope = if state.beep_timer < 0.01 {
                            state.beep_timer / 0.01
                        } else if state.beep_timer > 0.09 {
                            (0.1 - state.beep_timer) / 0.01
                        } else {
                            1.0
                        };

                        (time * state.beep_frequency * 2.0 * std::f32::consts::PI).sin()
                            * 0.3
                            * envelope
                    } else {
                        0.0
                    };

                    // Mix engine and beep sounds
                    let sample = (engine_sample + beep_sample).clamp(-1.0, 1.0);

                    // Write to all channels
                    for channel_sample in frame.iter_mut() {
                        *channel_sample = sample;
                    }
                }
            },
            |err| eprintln!("Audio stream error: {}", err),
            None,
        )?;

        Ok(stream)
    }

    fn sawtooth(time: f32, frequency: f32) -> f32 {
        2.0 * (time * frequency - (time * frequency + 0.5).floor()) - 1.0
    }

    /// Update engine sound based on RPM and throttle
    pub fn update_engine(&self, rpm: u32, throttle: f32, redline: u32) {
        if let Ok(mut state) = self.state.lock() {
            // Map RPM to frequency (typical engine sounds are in the 50-400 Hz range)
            // Lower RPMs = lower frequency, higher RPMs = higher frequency
            let rpm_ratio = rpm as f32 / redline as f32;
            state.engine_frequency = 50.0 + (rpm_ratio * 350.0);

            // Amplitude based on throttle (idle vs full throttle)
            if throttle > 0.0 {
                state.engine_amplitude = 0.15 + (throttle * 0.25);
            } else {
                // Idle sound
                state.engine_amplitude = if rpm > 0 { 0.08 } else { 0.0 };
            }
        }
    }

    /// Play a beep for Christmas tree lights
    pub fn play_beep(&self, beep_type: BeepType) {
        if let Ok(mut state) = self.state.lock() {
            state.beep_frequency = match beep_type {
                BeepType::Yellow => 800.0,   // Mid-range beep
                BeepType::Green => 1200.0,   // Higher pitch for green
                BeepType::RedLight => 400.0, // Lower pitch for red light
            };
            state.beep_active = true;
            state.beep_timer = 0.0;
        }
    }

    /// Update beep timer (call this in your update loop)
    pub fn update_beeps(&self, delta_time: f32) {
        if let Ok(mut state) = self.state.lock()
            && state.beep_active
        {
            state.beep_timer += delta_time;

            // Beep duration: 0.1 seconds
            if state.beep_timer >= 0.1 {
                state.beep_active = false;
                state.beep_timer = 0.0;
            }
        }
    }

    /// Stop all sounds
    pub fn stop(&self) {
        if let Ok(mut state) = self.state.lock() {
            state.engine_amplitude = 0.0;
            state.beep_active = false;
        }
    }

    /// Reset all audio state (useful when toggling mute)
    pub fn reset(&self) {
        if let Ok(mut state) = self.state.lock() {
            state.engine_amplitude = 0.0;
            state.engine_frequency = 0.0;
            state.beep_active = false;
            state.beep_timer = 0.0;
            state.beep_frequency = 0.0;
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum BeepType {
    Yellow,
    Green,
    RedLight,
}
