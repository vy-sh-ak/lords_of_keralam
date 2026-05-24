use bevy::{math::Rot2, prelude::*};

pub const WORLD_NORTH: Vec3 = Vec3::new(0.0, 0.0, -1.0);
pub const WORLD_EAST: Vec3 = Vec3::new(1.0, 0.0, 0.0);

#[derive(Debug, Resource)]
pub struct WorldDirection {
    pub north: Vec3,
    pub east: Vec3,
    pub heading_radians: f32,
}

impl Default for WorldDirection {
    fn default() -> Self {
        Self {
            north: WORLD_NORTH,
            east: WORLD_EAST,
            heading_radians: 0.0,
        }
    }
}

impl WorldDirection {
    pub fn from_orbit_yaw(orbit_yaw: f32) -> Self {
        let mut world_direction = Self::default();
        world_direction.set_heading_from_orbit_yaw(orbit_yaw);
        world_direction
    }

    pub fn compass_rotation(&self) -> Rot2 {
        // Rotate opposite the current heading so the needle keeps pointing north.
        Rot2::radians(-self.heading_radians)
    }

    pub fn set_heading_from_orbit_yaw(&mut self, orbit_yaw: f32) {
        let forward = Vec3::new(-orbit_yaw.cos(), 0.0, -orbit_yaw.sin());
        self.set_heading_from_forward(forward);
    }

    pub fn set_heading_from_forward(&mut self, forward: Vec3) {
        let horizontal_forward = Vec3::new(forward.x, 0.0, forward.z).normalize_or_zero();

        if horizontal_forward == Vec3::ZERO {
            return;
        }

        let east_component = horizontal_forward.dot(self.east);
        let north_component = horizontal_forward.dot(self.north);
        self.heading_radians = east_component.atan2(north_component);
    }
}