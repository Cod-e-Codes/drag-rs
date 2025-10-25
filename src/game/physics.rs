use super::Car;

const DRAG_COEFFICIENT: f64 = 0.3;
const AIR_DENSITY: f64 = 1.225;
const FRONTAL_AREA: f64 = 2.2;
const ROLLING_RESISTANCE: f64 = 0.015;
const GRAVITY: f64 = 9.81;

pub fn calculate_acceleration(
    car: &Car,
    velocity: f64,
    rpm: u32,
    gear: u8,
    throttle: f64,
    nos_active: bool,
    perfect_shift_boost: f64,
) -> f64 {
    if gear >= car.gear_ratios.len() as u8 {
        return 0.0;
    }

    let gear_ratio = car.gear_ratios[gear as usize];

    // Engine force based on RPM efficiency curve
    let rpm_efficiency = calculate_rpm_efficiency(rpm, car.redline);
    let base_torque = car.torque as f64 * rpm_efficiency * throttle;

    // Gear multiplication
    let wheel_torque = base_torque * gear_ratio;

    // Convert to force (simplified)
    let mut engine_force = wheel_torque * 3.5;

    // Nitrous boost
    if nos_active {
        engine_force *= 1.45;
    }

    // Perfect shift boost
    if perfect_shift_boost > 0.0 {
        engine_force *= 1.15;
    }

    // Drag force (increases with velocity squared)
    let drag_force = 0.5 * AIR_DENSITY * DRAG_COEFFICIENT * FRONTAL_AREA * velocity.powi(2);

    // Rolling resistance
    let rolling_force = ROLLING_RESISTANCE * car.weight as f64 * GRAVITY;

    // Net force and acceleration
    let net_force = engine_force - drag_force - rolling_force;
    net_force / car.weight as f64
}

pub fn calculate_rpm(velocity: f64, gear_ratio: f64) -> u32 {
    // Wheel RPM based on velocity
    let tire_diameter = 0.65; // meters
    let tire_circumference = std::f64::consts::PI * tire_diameter;
    let wheel_rpm = (velocity * 60.0) / tire_circumference;

    // Engine RPM
    let engine_rpm = wheel_rpm * gear_ratio;
    engine_rpm.max(800.0) as u32
}

fn calculate_rpm_efficiency(rpm: u32, redline: u32) -> f64 {
    let rpm_percent = rpm as f64 / redline as f64;

    // Power curve: peaks around 70-85% of redline
    if rpm_percent < 0.3 {
        0.4 + rpm_percent
    } else if rpm_percent < 0.7 {
        0.9 + (rpm_percent - 0.3) * 0.25
    } else if rpm_percent < 0.85 {
        1.0
    } else if rpm_percent < 0.95 {
        1.0 - (rpm_percent - 0.85) * 0.5
    } else {
        0.95 - (rpm_percent - 0.95) * 2.0
    }
}

pub fn calculate_shift_quality(rpm: u32, redline: u32) -> ShiftQuality {
    let optimal_start = (redline as f64 * 0.85) as u32;
    let optimal_end = (redline as f64 * 0.92) as u32;

    if rpm >= optimal_start && rpm <= optimal_end {
        ShiftQuality::Perfect
    } else if rpm >= optimal_start - 300 && rpm <= optimal_end + 300 {
        ShiftQuality::Good
    } else if rpm < 3000 {
        ShiftQuality::TooEarly
    } else {
        ShiftQuality::Missed
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ShiftQuality {
    Perfect,
    Good,
    Missed,
    TooEarly,
}
