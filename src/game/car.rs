#[derive(Debug, Clone)]
pub struct Car {
    pub name: String,
    pub horsepower: u32,
    pub weight: u32,
    pub torque: u32,
    pub redline: u32,
    pub gear_ratios: Vec<f64>,
}

impl Car {
    pub fn civic() -> Self {
        Self {
            name: "Honda Civic Si".to_string(),
            horsepower: 200,
            weight: 1300,
            torque: 192,
            redline: 8000,
            gear_ratios: vec![3.267, 1.967, 1.367, 1.033, 0.833],
        }
    }

    pub fn mustang() -> Self {
        Self {
            name: "Ford Mustang GT".to_string(),
            horsepower: 450,
            weight: 1700,
            torque: 410,
            redline: 7500,
            gear_ratios: vec![3.66, 2.43, 1.69, 1.32, 1.00],
        }
    }

    pub fn gtr() -> Self {
        Self {
            name: "Nissan GT-R".to_string(),
            horsepower: 565,
            weight: 1740,
            torque: 467,
            redline: 7000,
            gear_ratios: vec![4.056, 2.301, 1.595, 1.248, 1.001, 0.796],
        }
    }
}
