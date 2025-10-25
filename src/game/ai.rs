use super::{Car, RaceCarState};

pub struct AI {
    reaction_time: f64,
    has_launched: bool,
    shift_timing_variance: u32,
    nos_strategy: NosStrategy,
}

enum NosStrategy {
    Late, // Use in final gears
}

impl AI {
    pub fn new(reaction_time: f64) -> Self {
        Self {
            reaction_time,
            has_launched: false,
            shift_timing_variance: 200,
            nos_strategy: NosStrategy::Late,
        }
    }

    pub fn start_race(&mut self, state: &mut RaceCarState) {
        // AI starts with its reaction time
        state.reaction_time = Some(self.reaction_time);
        state.throttle = 1.0;
        self.has_launched = true;
    }

    pub fn update(&mut self, state: &mut RaceCarState, car: &Car, _delta_time: f64) {
        // AI doesn't launch until the race has actually started
        if !self.has_launched {
            return;
        }

        // Always on throttle
        state.throttle = 1.0;

        // Shift logic
        let optimal_shift_rpm = (car.redline as f64 * 0.88) as u32;
        let shift_target = optimal_shift_rpm + self.shift_timing_variance;

        if state.rpm >= shift_target && state.gear < car.gear_ratios.len() as u8 - 1 {
            state.shift_up(car);
        }

        // NOS strategy - use in final gears
        match self.nos_strategy {
            NosStrategy::Late => {
                if state.gear >= car.gear_ratios.len() as u8 - 2 && state.nos_remaining > 0.0 {
                    state.nos_active = true;
                }
            }
        }
    }
}
