use doryen_rs::Color;

pub mod ai;
pub mod input;
pub mod stats;

use crate::{CONSOLE_HEIGHT, CONSOLE_WIDTH};
pub use input::Player;

#[derive(Debug)]
pub enum DistanceMetric {
    /// Manhattan Distance (abs(dx) + abs(dy)). Use if you want things to be box like.
    Manhattan,
    /// Euclidean Distance. Slow to run, but use if you want things to be circular.
    Euclidean,
    /// Squared Euclidean Distance. Faster than Euclidean (cuz no sqrt) but you need to square your comparison. Can be used in the same way as Euclidean.
    EuclideanSquared,
}

/// World Coordinates
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Position {
    pub x: isize,
    pub y: isize,
}

impl DistanceMetric {
    pub fn distance(&self, pos: &Position, other: &Position) -> f64 {
        let out_distance = match self {
            DistanceMetric::Manhattan => pos.fast_distance(other),
            DistanceMetric::Euclidean => pos.euclidean_distance(other),
            DistanceMetric::EuclideanSquared => pos.distance_squared(other),
        };
        tracing::trace!(?out_distance, ?pos, ?other);
        out_distance
    }
}

impl Position {
    pub fn new(x: isize, y: isize) -> Position {
        Position { x, y }
    }

    pub fn go_towards(&self, other: &Position) -> Position {
        let angle = self.angle(other);
        let (dy, dx) = angle.sin_cos();
        let (adx, ady) = (dx.abs(), dy.abs());
        let (sdx, sdy) = (dx.signum(), dy.signum());
        let out_pos = if adx > ady {
            Position {
                x: self.x + sdx as isize * 1,
                y: self.y,
            }
        } else if ady > adx {
            Position {
                x: self.x,
                y: self.y + sdy as isize * 1,
            }
        } else {
            // They're both equal so let's go diagonally.
            Position {
                x: self.x + 1 * sdx as isize,
                y: self.y + 1 * sdy as isize,
            }
        };
        tracing::trace!(?out_pos, ?other, ?self, ?angle, ?dy, ?dx);
        out_pos
        // self.go_distance_theta(distance as f64, angle)
    }

    /// Inclusive bounds.
    pub fn is_within_bounds(&self, (min_x, max_x): (u32, u32), (min_y, max_y): (u32, u32)) -> bool {
        self.x >= min_x as isize
            && self.x <= max_x as isize
            && self.y >= min_y as isize
            && self.y <= max_y as isize
    }

    pub fn is_within_console_bounds(&self) -> bool {
        self.is_within_bounds((1, CONSOLE_WIDTH - 2), (1, CONSOLE_HEIGHT - 2))
    }

    pub fn go_distance_theta(&self, distance: f64, theta: f64) -> Position {
        let (dx, dy) = theta.sin_cos();
        let x = (dx * distance) as isize;
        let y = (dy * distance) as isize;
        let out = Position::new(x + self.x, y + self.y);
        tracing::trace!(?out, ?self, ?distance, ?theta, ?dy, ?dx);
        out
    }

    pub fn new_from_dx_dy(&self, dx: isize, dy: isize) -> Position {
        Position::new(self.x + dx, self.y + dy)
    }

    /// Output is in radians. Uses Manhattan distance by default.
    pub fn angle(&self, other: &Position) -> f64 {
        let dx = (other.x - self.x) as f64;
        let dy = (other.y - self.y) as f64;
        let theta = dy.atan2(dx);
        tracing::trace!(?theta, ?self, ?other);
        theta
    }

    pub fn distance(&self, other: &Position, method: &DistanceMetric) -> f64 {
        method.distance(self, other)
    }

    pub fn distance_from_zero(&self, method: &DistanceMetric) -> f64 {
        self.distance(&ZERO_POS, method)
    }

    fn dot_product(&self, other: &Position) -> isize {
        let product = self.x * other.x + self.y * other.y;
        tracing::debug!(?product, ?self, ?other);
        product
    }

    /// Manhattan distance
    pub fn fast_distance(&self, other: &Position) -> f64 {
        let (dx, dy) = (self.x - other.x, self.y - other.y);
        (dx.abs() + dy.abs()) as f64
    }

    /// Euclidean distance squared.
    pub fn distance_squared(&self, other: &Position) -> f64 {
        let (dx, dy) = (
            self.x as f64 - other.x as f64,
            self.y as f64 - other.y as f64,
        );
        dx.powi(2) + dy.powi(2)
    }

    /// Euclidean Distance (fr fr)
    pub fn euclidean_distance(&self, other: &Position) -> f64 {
        self.distance_squared(other).sqrt()
    }
}

pub const ZERO_POS: Position = Position { x: 0, y: 0 };

/// World Coordinates
#[derive(Debug)]
pub struct WindowCoordinates {
    x: f64,
    y: f64,
}

#[derive(Debug)]
pub struct Motion;

#[derive(Debug)]
pub struct Renderable {
    pub glyph: char,
    pub color: Color,
}

mod tests {
    use super::*;
    // use crate::models::Position;

    #[test]
    fn test_position_create() {
        let pos = Position::new(10, 10);
        assert_eq!(pos.x, 10);
        assert_eq!(pos.y, 10);
    }

    #[test]
    fn test_position_fast_distance() {
        let one = Position::new(0, 10);
        let two = Position::new(10, 10);
        assert_eq!(one.fast_distance(&one), 0.0);
        assert_eq!(two.fast_distance(&two), 0.0);
        let distance = one.fast_distance(&two);
        assert_eq!(distance, 10.0);
        assert_eq!(distance, two.fast_distance(&one));

        let three = Position::new(0, 0);
        let distance = two.fast_distance(&three);
        assert_eq!(distance, 20.0);
        assert_eq!(distance, three.fast_distance(&two));

        let distance_one_two = one.fast_distance(&two);
        let distance_two_three = two.fast_distance(&three);
        let distance_one_three = one.fast_distance(&three);
        assert!(distance_one_two + distance_two_three >= distance_one_three);
    }

    #[test]
    fn test_position_euclidean_distance_squared() {
        let one = Position::new(0, 0);
        let two = Position::new(10, 0);
        // Check Axiom 1 here - https://en.wikipedia.org/wiki/Metric_space#Definition
        assert_eq!(two.euclidean_distance(&two), 0.0);
        assert_eq!(one.euclidean_distance(&one), 0.0);
        let distance = two.distance_squared(&one);
        assert_eq!(distance, 100.0);
        // Check symmetry: https://en.wikipedia.org/wiki/Metric_space#Definition
        assert_eq!(distance, one.distance_squared(&two));

        let three = Position::new(10, 10);
        assert_eq!(three.euclidean_distance(&three), 0.0);
        let distance = three.distance_squared(&one);
        assert_eq!(distance, 200.0);
        assert_eq!(distance, one.distance_squared(&three));

        // Check triangle inequality: https://en.wikipedia.org/wiki/Metric_space#Definition
        let distance_one_two = one.distance_squared(&two);
        let distance_two_three = two.distance_squared(&three);
        let distance_one_three = one.distance_squared(&three);
        // println!("distance 1->2: {distance_one_two}");
        // println!("distance 2->3: {distance_two_three}");
        // println!("distance 1->3: {distance_one_three}");
        assert!(distance_one_two + distance_two_three >= distance_one_three);
    }

    #[test]
    fn test_position_euclidean_distance() {
        let one = Position::new(0, 0);
        let two = Position::new(10, 0);
        assert_eq!(two.euclidean_distance(&two), 0.0);
        assert_eq!(one.euclidean_distance(&one), 0.0);

        let distance = two.euclidean_distance(&one);
        assert_eq!(distance, 10.0);
        assert_eq!(distance, one.euclidean_distance(&two));

        let three = Position::new(10, 10);
        assert_eq!(three.euclidean_distance(&three), 0.0);
        let distance = three.euclidean_distance(&one);
        assert_eq!(distance, 10.0 * 2.0_f64.sqrt());

        let distance_one_two = one.euclidean_distance(&two);
        let distance_two_three = two.euclidean_distance(&three);
        let distance_one_three = one.euclidean_distance(&three);
        assert!(distance_one_two + distance_two_three >= distance_one_three);
    }

    #[test]
    fn test_distance_metric() {
        // We've already tested all the underlying distance functions
        // so this one is more just to make sure that we get the correct distances
        // for each DistanceMetric
        let one = Position::new(0, 0);
        let two = Position::new(10, 10);

        let distance = one.distance(&two, &DistanceMetric::Manhattan);
        assert_eq!(distance, 20.0);
        assert_eq!(distance, DistanceMetric::Manhattan.distance(&one, &two));

        let distance = one.distance(&two, &DistanceMetric::Euclidean);
        assert_eq!(distance, 10.0 * 2.0_f64.sqrt());
        assert_eq!(distance, DistanceMetric::Euclidean.distance(&one, &two));

        let distance = one.distance(&two, &DistanceMetric::EuclideanSquared);
        assert_eq!(distance, 200.0);
        assert_eq!(
            distance,
            DistanceMetric::EuclideanSquared.distance(&one, &two)
        );
    }

    #[test]
    fn test_position_angle() {
        let one = Position::new(0, 0);
        let two = Position::new(1, 1);

        // 45 degrees
        let angle = one.angle(&two);
        assert_eq!(angle, 45.0_f64.to_radians());
        let angle2 = two.angle(&one);
        assert_eq!(angle2, -135.0_f64.to_radians());

        // 90 degrees
        let two = Position::new(0, 1);
        let angle = one.angle(&two);
        assert_eq!(angle, 90.0_f64.to_radians());
        let angle2 = two.angle(&one);
        assert_eq!(angle2, -90.0_f64.to_radians());

        // 0 degrees
        let two = Position::new(1, 0);
        let angle = one.angle(&two);
        assert_eq!(angle, 0.0);
        let angle2 = two.angle(&one);
        assert_eq!(angle2, 180.0_f64.to_radians());
    }

    #[test]
    fn test_go_towards() {
        let initial_start = Position::new(0, 0);
        let goal = Position::new(10, 10);

        let mut curr_pos = initial_start;

        while curr_pos.distance(&goal, &DistanceMetric::Euclidean) > 0.0 {
            let next_pos = curr_pos.go_towards(&goal);
            println!("Curr Pos {curr_pos:?} -> Goal {goal:?} => Next Pos {next_pos:?}");
            let distance_to_goal = curr_pos.distance(&goal, &DistanceMetric::Euclidean);
            let next_distance_to_goal = next_pos.distance(&goal, &DistanceMetric::Euclidean);
            println!("Distance to goal: {distance_to_goal}");
            println!("Next Distance to goal: {next_distance_to_goal}");

            assert!(distance_to_goal > next_distance_to_goal);
            curr_pos = next_pos;
        }
    }
}
