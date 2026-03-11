use std::collections::HashMap;

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::rocket::{RocketFlightParameters, RocketMassModel};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MotorSize {
    Small,
    Medium,
    Large,
}

impl MotorSize {
    pub const ALL: [MotorSize; 3] = [Self::Small, Self::Medium, Self::Large];

    pub fn label(self) -> &'static str {
        match self {
            Self::Small => "Small",
            Self::Medium => "Medium",
            Self::Large => "Large",
        }
    }

    pub fn price(self) -> f64 {
        match self {
            Self::Small => 2.0,
            Self::Medium => 5.0,
            Self::Large => 12.0,
        }
    }

    pub fn pack_price(self) -> f64 {
        self.price() * 3.0
    }

    pub fn unlock_price(self) -> f64 {
        match self {
            Self::Small => 0.0,
            Self::Medium => 15.0,
            Self::Large => 30.0,
        }
    }

    pub fn flight_parameters(self) -> RocketFlightParameters {
        match self {
            Self::Small => RocketFlightParameters {
                force: 4.0,
                duration: 1.0,
            },
            Self::Medium => RocketFlightParameters {
                force: 7.0,
                duration: 1.5,
            },
            Self::Large => RocketFlightParameters {
                force: 12.0,
                duration: 2.0,
            },
        }
    }

    pub fn motor_mass_kg(self) -> f32 {
        match self {
            Self::Small => 0.040,
            Self::Medium => 0.080,
            Self::Large => 0.150,
        }
    }

    pub fn motor_length_m(self) -> f32 {
        match self {
            Self::Small => 0.070,
            Self::Medium => 0.095,
            Self::Large => 0.120,
        }
    }

    pub fn apply_to_mass_model(self, model: &mut RocketMassModel) {
        model.motor_mass_kg = self.motor_mass_kg();
        model.motor_length_m = self.motor_length_m();
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ParachuteSize {
    Small,
    Large,
}

impl ParachuteSize {
    pub const ALL: [ParachuteSize; 2] = [Self::Small, Self::Large];

    pub fn label(self) -> &'static str {
        match self {
            Self::Small => "Small",
            Self::Large => "Large",
        }
    }

    pub fn price(self) -> f64 {
        match self {
            Self::Small => 5.0,
            Self::Large => 12.0,
        }
    }

    pub fn diameter(self) -> f32 {
        match self {
            Self::Small => 0.3,
            Self::Large => 0.6,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TubeType {
    Standard,
    Reinforced,
    Lightweight,
}

impl TubeType {
    pub const ALL: [TubeType; 3] = [Self::Standard, Self::Reinforced, Self::Lightweight];

    pub fn label(self) -> &'static str {
        match self {
            Self::Standard => "Standard",
            Self::Reinforced => "Reinforced",
            Self::Lightweight => "Lightweight",
        }
    }

    pub fn price(self) -> f64 {
        match self {
            Self::Standard => 0.0,
            Self::Reinforced => 10.0,
            Self::Lightweight => 15.0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum NoseconeType {
    Ogive,
    Conical,
    Elliptical,
}

impl NoseconeType {
    pub const ALL: [NoseconeType; 3] = [Self::Ogive, Self::Conical, Self::Elliptical];

    pub fn label(self) -> &'static str {
        match self {
            Self::Ogive => "Ogive",
            Self::Conical => "Conical",
            Self::Elliptical => "Elliptical",
        }
    }

    pub fn price(self) -> f64 {
        match self {
            Self::Ogive => 0.0,
            Self::Conical => 8.0,
            Self::Elliptical => 12.0,
        }
    }
}

#[derive(Resource, Clone, Serialize, Deserialize)]
pub struct Inventory {
    pub motors: HashMap<MotorSize, u32>,
    pub parachutes: HashMap<ParachuteSize, u32>,
}

impl Default for Inventory {
    fn default() -> Self {
        let mut motors = HashMap::new();
        motors.insert(MotorSize::Small, 3);
        let mut parachutes = HashMap::new();
        parachutes.insert(ParachuteSize::Small, 1);
        Self { motors, parachutes }
    }
}

impl Inventory {
    pub fn motor_count(&self, size: MotorSize) -> u32 {
        self.motors.get(&size).copied().unwrap_or(0)
    }

    pub fn parachute_count(&self, size: ParachuteSize) -> u32 {
        self.parachutes.get(&size).copied().unwrap_or(0)
    }

    pub fn consume_motor(&mut self, size: MotorSize) -> bool {
        let count = self.motors.entry(size).or_insert(0);
        if *count > 0 {
            *count -= 1;
            true
        } else {
            false
        }
    }

    pub fn add_motors(&mut self, size: MotorSize, qty: u32) {
        *self.motors.entry(size).or_insert(0) += qty;
    }
}

#[derive(Resource, Clone, Serialize, Deserialize)]
pub struct OwnedMotorSizes(pub Vec<MotorSize>);

impl Default for OwnedMotorSizes {
    fn default() -> Self {
        Self(vec![MotorSize::Small])
    }
}

#[derive(Resource, Clone, Serialize, Deserialize)]
pub struct OwnedTubeTypes(pub Vec<TubeType>);

impl Default for OwnedTubeTypes {
    fn default() -> Self {
        Self(vec![TubeType::Standard])
    }
}

#[derive(Resource, Clone, Serialize, Deserialize)]
pub struct OwnedNoseconeTypes(pub Vec<NoseconeType>);

impl Default for OwnedNoseconeTypes {
    fn default() -> Self {
        Self(vec![NoseconeType::Ogive])
    }
}

#[derive(Resource, Clone, Serialize, Deserialize)]
pub struct EquippedLoadout {
    pub motor: MotorSize,
    pub parachute: ParachuteSize,
    pub tube_type: TubeType,
    pub nosecone_type: NoseconeType,
}

impl Default for EquippedLoadout {
    fn default() -> Self {
        Self {
            motor: MotorSize::Small,
            parachute: ParachuteSize::Small,
            tube_type: TubeType::Standard,
            nosecone_type: NoseconeType::Ogive,
        }
    }
}

#[derive(Resource, Clone, Serialize, Deserialize, Default)]
pub struct PlayerExperience(pub u64);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_inventory_has_starting_items() {
        let inv = Inventory::default();
        assert_eq!(inv.motor_count(MotorSize::Small), 3);
        assert_eq!(inv.motor_count(MotorSize::Medium), 0);
        assert_eq!(inv.parachute_count(ParachuteSize::Small), 1);
        assert_eq!(inv.parachute_count(ParachuteSize::Large), 0);
    }

    #[test]
    fn consume_motor_decrements_and_fails_at_zero() {
        let mut inv = Inventory::default();
        assert!(inv.consume_motor(MotorSize::Small));
        assert_eq!(inv.motor_count(MotorSize::Small), 2);
        assert!(inv.consume_motor(MotorSize::Small));
        assert!(inv.consume_motor(MotorSize::Small));
        assert!(!inv.consume_motor(MotorSize::Small));
        assert_eq!(inv.motor_count(MotorSize::Small), 0);
    }

    #[test]
    fn add_motors_works() {
        let mut inv = Inventory::default();
        inv.add_motors(MotorSize::Medium, 3);
        assert_eq!(inv.motor_count(MotorSize::Medium), 3);
        inv.add_motors(MotorSize::Medium, 3);
        assert_eq!(inv.motor_count(MotorSize::Medium), 6);
    }

    #[test]
    fn motor_pack_price_is_3x_unit() {
        for size in MotorSize::ALL {
            assert!((size.pack_price() - size.price() * 3.0).abs() < f64::EPSILON);
        }
    }
}
