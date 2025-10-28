use doryen_rs::Color;

pub mod ai;
pub mod input;

pub use input::Player;

pub enum DistanceMetric {
    Manhattan,
    Euclidean,
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
        match self {
            DistanceMetric::Manhattan => pos.fast_distance(other),
            DistanceMetric::Euclidean => pos.euclidean_distance(other),
            DistanceMetric::EuclideanSquared => pos.distance_squared(other),
        }
    }
}

impl Position {
    pub fn new(x: isize, y: isize) -> Position {
        Position { x, y }
    }

    pub fn go_towards(&self, other: &Position, distance: u32) -> Position {
        let angle = self.angle(other, None);
        self.go_distance_theta(distance as f64, angle)
    }

    pub fn go_distance_theta(&self, distance: f64, theta: f64) -> Position {
        let (dx, dy) = theta.sin_cos();
        let x = (dx * distance) as isize;
        let y = (dy * distance) as isize;
        Position::new(x + self.x, y + self.y)
    }

    /// Output is in radians. Uses Manhattan distance by default.
    pub fn angle(&self, other: &Position, distance_metric: Option<DistanceMetric>) -> f64 {
        let metric = distance_metric.unwrap_or(DistanceMetric::Manhattan);
        let self_magnitude = self.distance_from_zero(&metric);
        let other_magnitude = other.distance_from_zero(&metric);
        let dot = self.dot_product(other) as f64;
        let inner = dot / (self_magnitude * other_magnitude);
        inner.acos()
    }

    pub fn distance(&self, other: &Position, method: &DistanceMetric) -> f64 {
        method.distance(self, other)
    }

    pub fn distance_from_zero(&self, method: &DistanceMetric) -> f64 {
        self.distance(&ZERO_POS, method)
    }

    fn dot_product(&self, other: &Position) -> isize {
        self.x * other.x + self.y * other.y
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
pub struct Health {
    pub total_health: u32,
    pub current_health: i32,
}

impl Health {
    pub fn new(health: u32) -> Health {
        Health {
            total_health: health,
            current_health: health as i32,
        }
    }
    pub fn get_ratio(&self) -> f32 {
        self.current_health as f32 / self.total_health as f32
    }
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
        assert_eq!(distance, DistanceMetric::EuclideanSquared.distance(&one, &two));
    }
}
