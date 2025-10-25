use crate::game::{Car, RaceState};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AppState {
    Menu,
    Racing,
    Results,
}

pub struct App {
    pub state: AppState,
    pub race_state: Option<RaceState>,
    pub player_car: Car,
    pub opponent_car: Car,
    pub should_quit: bool,
    pub selected_car_index: usize,
    pub key_states: KeyStates,
    pub audio_muted: bool,
}

#[derive(Debug, Clone)]
pub struct KeyStates {
    pub throttle_pressed: bool,
    pub nitrous_pressed: bool,
    pub shift_pressed: bool,
}

impl App {
    pub fn new() -> Self {
        Self {
            state: AppState::Menu,
            race_state: None,
            player_car: Car::civic(),
            opponent_car: Car::mustang(), // Different opponent car
            should_quit: false,
            selected_car_index: 0,
            key_states: KeyStates {
                throttle_pressed: false,
                nitrous_pressed: false,
                shift_pressed: false,
            },
            audio_muted: false,
        }
    }

    pub fn start_race(&mut self) {
        self.reset_all_key_states();
        self.race_state = Some(RaceState::new(
            self.player_car.clone(),
            self.opponent_car.clone(),
        ));
        self.state = AppState::Racing;
    }

    pub fn update(&mut self, delta_time: f64) {
        // Handle continuous key states
        if let Some(race) = &mut self.race_state {
            // Update throttle based on key state
            if self.key_states.throttle_pressed {
                race.player_throttle();
            } else {
                race.player_release_throttle();
            }

            // Update nitrous based on key state
            if self.key_states.nitrous_pressed {
                race.player_activate_nos();
            } else {
                race.player_deactivate_nos();
            }

            race.update(delta_time);

            if race.is_finished() {
                self.state = AppState::Results;
            }
        }
    }

    pub fn set_throttle_pressed(&mut self, pressed: bool) {
        self.key_states.throttle_pressed = pressed;
    }

    pub fn set_nitrous_pressed(&mut self, pressed: bool) {
        self.key_states.nitrous_pressed = pressed;
    }

    pub fn shift_up(&mut self) {
        if !self.key_states.shift_pressed {
            self.key_states.shift_pressed = true;
            if let Some(race) = &mut self.race_state {
                race.player_shift_up();
            }
        }
    }

    pub fn reset_shift_state(&mut self) {
        self.key_states.shift_pressed = false;
    }

    pub fn get_available_cars() -> Vec<Car> {
        vec![Car::civic(), Car::mustang(), Car::gtr()]
    }

    pub fn select_next_car(&mut self) {
        let cars = Self::get_available_cars();
        self.selected_car_index = (self.selected_car_index + 1) % cars.len();
        self.player_car = cars[self.selected_car_index].clone();
    }

    pub fn select_previous_car(&mut self) {
        let cars = Self::get_available_cars();
        self.selected_car_index = if self.selected_car_index == 0 {
            cars.len() - 1
        } else {
            self.selected_car_index - 1
        };
        self.player_car = cars[self.selected_car_index].clone();
    }

    pub fn reset_all_key_states(&mut self) {
        self.key_states.throttle_pressed = false;
        self.key_states.nitrous_pressed = false;
        self.key_states.shift_pressed = false;
    }

    pub fn toggle_mute(&mut self) {
        self.audio_muted = !self.audio_muted;
    }
}
