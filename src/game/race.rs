use super::physics::{
    ShiftQuality, calculate_acceleration, calculate_rpm, calculate_shift_quality,
};
use super::{AI, Car};
use std::time::Instant;

const FINISH_LINE: f64 = 402.336; // Quarter mile in meters

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LightState {
    PreStage,
    Staged,
    Yellow1,
    Yellow2,
    Yellow3,
    Green,
    Racing,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Winner {
    Player,
    Opponent,
}

pub struct ChristmasTree {
    pub state: LightState,
    state_timer: f64,
}

impl ChristmasTree {
    fn new() -> Self {
        Self {
            state: LightState::PreStage,
            state_timer: 0.0,
        }
    }

    fn update(&mut self, delta_time: f64) -> bool {
        self.state_timer += delta_time;

        match self.state {
            LightState::PreStage if self.state_timer >= 0.5 => {
                self.state = LightState::Staged;
                self.state_timer = 0.0;
            }
            LightState::Staged if self.state_timer >= 0.5 => {
                self.state = LightState::Yellow1;
                self.state_timer = 0.0;
            }
            LightState::Yellow1 if self.state_timer >= 0.5 => {
                self.state = LightState::Yellow2;
                self.state_timer = 0.0;
            }
            LightState::Yellow2 if self.state_timer >= 0.5 => {
                self.state = LightState::Yellow3;
                self.state_timer = 0.0;
            }
            LightState::Yellow3 if self.state_timer >= 0.5 => {
                self.state = LightState::Green;
                self.state_timer = 0.0;
                return true; // Race can start
            }
            // Don't auto-transition to Racing - let the race logic handle this
            _ => {}
        }
        false
    }
}

#[derive(Debug, Clone)]
pub struct RaceCarState {
    pub position: f64,
    pub velocity: f64,
    pub rpm: u32,
    pub gear: u8,
    pub throttle: f64,
    pub nos_remaining: f64,
    pub nos_active: bool,
    pub engine_heat: f64,
    pub reaction_time: Option<f64>,
    pub finish_time: Option<f64>,
    pub top_speed: f64,
    pub perfect_shifts: u8,
    pub blown_engine: bool,
    pub perfect_shift_boost: f64,
    pub last_shift_quality: Option<ShiftQuality>,
}

impl RaceCarState {
    fn new() -> Self {
        Self {
            position: 0.0,
            velocity: 0.0,
            rpm: 1000,
            gear: 0,
            throttle: 0.0,
            nos_remaining: 10.0,
            nos_active: false,
            engine_heat: 0.0,
            reaction_time: None,
            finish_time: None,
            top_speed: 0.0,
            perfect_shifts: 0,
            blown_engine: false,
            perfect_shift_boost: 0.0,
            last_shift_quality: None,
        }
    }

    pub fn shift_up(&mut self, car: &Car) {
        if self.gear >= car.gear_ratios.len() as u8 - 1 {
            return;
        }

        let quality = calculate_shift_quality(self.rpm, car.redline);
        self.last_shift_quality = Some(quality);

        if quality == ShiftQuality::Perfect {
            self.perfect_shifts += 1;
            self.perfect_shift_boost = 0.5;
        }

        self.gear += 1;
        self.rpm = (self.rpm as f64 * 0.65) as u32;
    }
}

pub struct RaceState {
    pub player: RaceCarState,
    pub opponent: RaceCarState,
    pub player_car: Car,
    pub opponent_car: Car,
    pub christmas_tree: ChristmasTree,
    pub race_started: bool,
    pub race_finished: bool,
    pub winner: Option<Winner>,
    pub elapsed_time: f64,
    pub ai: AI,
    green_light_time: Option<Instant>,
}

impl RaceState {
    pub fn new(player_car: Car, opponent_car: Car) -> Self {
        Self {
            player: RaceCarState::new(),
            opponent: RaceCarState::new(),
            player_car,
            opponent_car,
            christmas_tree: ChristmasTree::new(),
            race_started: false,
            race_finished: false,
            winner: None,
            elapsed_time: 0.0,
            ai: AI::new(0.15), // Medium difficulty
            green_light_time: None,
        }
    }

    pub fn update(&mut self, delta_time: f64) {
        if self.race_finished {
            return;
        }

        // Update countdown
        if !self.race_started {
            if self.christmas_tree.update(delta_time) {
                self.green_light_time = Some(Instant::now());
            }
            return;
        }

        self.elapsed_time += delta_time;

        // Update player
        let player_car = self.player_car.clone();
        Self::update_car_static(&player_car, &mut self.player, delta_time);

        // Update opponent with AI
        let opponent_car = self.opponent_car.clone();
        self.ai
            .update(&mut self.opponent, &opponent_car, delta_time);
        Self::update_car_static(&opponent_car, &mut self.opponent, delta_time);

        // Check for finish
        if self.player.position >= FINISH_LINE && self.player.finish_time.is_none() {
            self.player.finish_time = Some(self.elapsed_time);
            if self.winner.is_none() {
                self.winner = Some(Winner::Player);
            }
        }

        if self.opponent.position >= FINISH_LINE && self.opponent.finish_time.is_none() {
            self.opponent.finish_time = Some(self.elapsed_time);
            if self.winner.is_none() {
                self.winner = Some(Winner::Opponent);
            }
        }

        if self.player.finish_time.is_some() && self.opponent.finish_time.is_some() {
            self.race_finished = true;
        }
    }

    fn update_car_static(car: &Car, state: &mut RaceCarState, delta_time: f64) {
        if state.blown_engine || state.finish_time.is_some() {
            return;
        }

        // Decay perfect shift boost
        if state.perfect_shift_boost > 0.0 {
            state.perfect_shift_boost -= delta_time;
            state.perfect_shift_boost = state.perfect_shift_boost.max(0.0);
        }

        // Calculate acceleration
        let acceleration = calculate_acceleration(
            car,
            state.velocity,
            state.rpm,
            state.gear,
            state.throttle,
            state.nos_active,
            state.perfect_shift_boost,
        );

        // Update velocity and position
        state.velocity += acceleration * delta_time;
        state.velocity = state.velocity.max(0.0);
        state.position += state.velocity * delta_time;

        // Track top speed
        if state.velocity > state.top_speed {
            state.top_speed = state.velocity;
        }

        // Update RPM based on velocity and gear
        if state.gear < car.gear_ratios.len() as u8 {
            let gear_ratio = car.gear_ratios[state.gear as usize];
            state.rpm = calculate_rpm(state.velocity, gear_ratio);
        }

        // Engine heat management
        if state.rpm > (car.redline as f64 * 0.9) as u32 {
            state.engine_heat += delta_time * 0.3;
        } else {
            state.engine_heat -= delta_time * 0.1;
        }
        state.engine_heat = state.engine_heat.clamp(0.0, 1.0);

        // Blown engine
        if state.engine_heat >= 1.0 {
            state.blown_engine = true;
            state.velocity = 0.0;
        }

        // NOS management
        if state.nos_active {
            state.nos_remaining -= delta_time;
            if state.nos_remaining <= 0.0 {
                state.nos_remaining = 0.0;
                state.nos_active = false;
            }
        }

        // Auto-shift if over redline (safety)
        if state.rpm > car.redline && state.gear < car.gear_ratios.len() as u8 - 1 {
            state.shift_up(car);
        }
    }

    pub fn player_throttle(&mut self) {
        if !self.race_started {
            match self.christmas_tree.state {
                LightState::Green => {
                    // Calculate reaction time
                    if let Some(green_time) = self.green_light_time {
                        let reaction = green_time.elapsed().as_secs_f64();
                        self.player.reaction_time = Some(reaction);
                        self.race_started = true;
                        
                        // Transition to Racing state
                        self.christmas_tree.state = LightState::Racing;
                        
                        // Start the AI opponent
                        self.ai.start_race(&mut self.opponent);
                    }
                }
                LightState::Yellow1 | LightState::Yellow2 | LightState::Yellow3 => {
                    // Red light - jumped the start during yellow lights!
                    self.player.reaction_time = Some(-1.0); // Negative indicates red light
                    self.race_started = true;
                    self.race_finished = true;
                    self.winner = Some(Winner::Opponent);
                    
                    // Transition to Racing state to show red light
                    self.christmas_tree.state = LightState::Racing;
                    return;
                }
                _ => {
                    // Pre-stage or staged - no penalty, just ignore
                    return;
                }
            }
        }

        if self.race_started {
            self.player.throttle = 1.0;
        }
    }

    pub fn player_release_throttle(&mut self) {
        if self.race_started {
            self.player.throttle = 0.0;
        }
    }

    pub fn player_shift_up(&mut self) {
        if self.race_started {
            self.player.shift_up(&self.player_car.clone());
        }
    }

    pub fn player_activate_nos(&mut self) {
        if self.race_started && self.player.nos_remaining > 0.0 {
            self.player.nos_active = true;
        }
    }

    pub fn player_deactivate_nos(&mut self) {
        if self.race_started {
            self.player.nos_active = false;
        }
    }

    pub fn is_finished(&self) -> bool {
        self.race_finished
    }

    pub fn get_player_progress(&self) -> f64 {
        (self.player.position / FINISH_LINE).min(1.0)
    }

    pub fn get_opponent_progress(&self) -> f64 {
        (self.opponent.position / FINISH_LINE).min(1.0)
    }
}
