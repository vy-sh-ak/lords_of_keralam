use bevy::prelude::*;
use bevy_inspector_egui::prelude::*;
use serde::{Deserialize, Serialize};

const MIN_POINT_COUNT: usize = 2;
const DEFAULT_MIDPOINT_INPUT: f32 = 0.5;
const DEFAULT_MIDPOINT_OUTPUT: f32 = 0.5;

#[derive(Reflect, InspectorOptions, Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(default)]
#[reflect(InspectorOptions)]
pub struct HeightCurve {
    pub points: Vec<HeightCurvePoint>,
}

#[derive(Reflect, InspectorOptions, Clone, Copy, Debug, Serialize, Deserialize, PartialEq)]
#[reflect(InspectorOptions)]
pub struct HeightCurvePoint {
    #[inspector(min = 0.0, max = 1.0)]
    pub input: f32,
    #[inspector(min = 0.0, max = 1.0)]
    pub output: f32,
}

impl Default for HeightCurve {
    fn default() -> Self {
        Self {
            points: vec![
                HeightCurvePoint::new(0.0, 0.0),
                HeightCurvePoint::new(DEFAULT_MIDPOINT_INPUT, DEFAULT_MIDPOINT_OUTPUT),
                HeightCurvePoint::new(1.0, 1.0),
            ],
        }
    }
}

impl HeightCurve {
    pub fn sample(&self, input: f32) -> f32 {
        let points = &self.points;
        if points.is_empty() {
            return input.clamp(0.0, 1.0);
        }

        let clamped_input = input.clamp(0.0, 1.0);
        if clamped_input <= points[0].input {
            return points[0].output;
        }

        for window in points.windows(2) {
            let start = window[0];
            let end = window[1];
            if clamped_input <= end.input {
                let span = end.input - start.input;
                if span.abs() <= f32::EPSILON {
                    return end.output;
                }

                let t = (clamped_input - start.input) / span;
                return start.output + (end.output - start.output) * t;
            }
        }

        points.last().map(|point| point.output).unwrap_or(clamped_input)
    }

    pub fn sanitized(&self) -> Self {
        let mut points: Vec<_> = self
            .points
            .iter()
            .copied()
            .filter(|point| point.input.is_finite() && point.output.is_finite())
            .map(|point| {
                HeightCurvePoint::new(point.input.clamp(0.0, 1.0), point.output.clamp(0.0, 1.0))
            })
            .collect();

        if points.len() < MIN_POINT_COUNT {
            return Self::default();
        }

        points.sort_by(|left, right| left.input.total_cmp(&right.input));
        points.dedup_by(|left, right| approx_eq_f32(left.input, right.input));

        if points.len() < MIN_POINT_COUNT {
            return Self::default();
        }

        if let Some(first) = points.first_mut() {
            first.input = 0.0;
        }

        if let Some(last) = points.last_mut() {
            last.input = 1.0;
        }

        Self { points }
    }

    pub fn set_point_input(&mut self, index: usize, input: f32) -> bool {
        if index == 0 || index + 1 >= self.points.len() || !input.is_finite() {
            return false;
        }

        let previous = self.points[index - 1].input;
        let next = self.points[index + 1].input;
        let clamped = input.clamp(previous, next);

        if approx_eq_f32(self.points[index].input, clamped) {
            return false;
        }

        self.points[index].input = clamped;
        true
    }

    pub fn set_point_output(&mut self, index: usize, output: f32) -> bool {
        let Some(point) = self.points.get_mut(index) else {
            return false;
        };

        if !output.is_finite() {
            return false;
        }

        let clamped = output.clamp(0.0, 1.0);
        if approx_eq_f32(point.output, clamped) {
            return false;
        }

        point.output = clamped;
        true
    }

    pub fn add_point(&mut self) -> bool {
        let insert_index = self.find_largest_gap_index();
        let start = self.points[insert_index];
        let end = self.points[insert_index + 1];
        let midpoint = HeightCurvePoint::new(
            (start.input + end.input) * 0.5,
            (start.output + end.output) * 0.5,
        );

        self.points.insert(insert_index + 1, midpoint);
        true
    }

    pub fn remove_point(&mut self, index: usize) -> bool {
        if self.points.len() <= MIN_POINT_COUNT || index == 0 || index + 1 == self.points.len() {
            return false;
        }

        self.points.remove(index);
        true
    }

    pub fn reset(&mut self) -> bool {
        if *self == Self::default() {
            return false;
        }

        *self = Self::default();
        true
    }

    fn find_largest_gap_index(&self) -> usize {
        self.points
            .windows(2)
            .enumerate()
            .max_by(|(_, left), (_, right)| {
                let left_gap = left[1].input - left[0].input;
                let right_gap = right[1].input - right[0].input;
                left_gap.total_cmp(&right_gap)
            })
            .map(|(index, _)| index)
            .unwrap_or(0)
    }
}

impl HeightCurvePoint {
    pub const fn new(input: f32, output: f32) -> Self {
        Self { input, output }
    }
}

fn approx_eq_f32(left: f32, right: f32) -> bool {
    (left - right).abs() <= f32::EPSILON
}