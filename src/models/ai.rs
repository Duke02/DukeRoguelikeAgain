use crate::models::{DistanceMetric, Health, Position};

#[derive(Debug)]
pub struct Vision {
    view_range: usize,
}

impl Vision {
    pub fn new(view_range: usize) -> Self {
        Vision { view_range }
    }
    pub fn can_see(&self, self_pos: &Position, position: &Position) -> bool {
        self_pos.distance(position, &DistanceMetric::EuclideanSquared)
            <= (self.view_range.pow(2) as f64)
    }
}

#[derive(Debug)]
pub enum Action {
    GoTo(Position),
    Wait,
    Attack(Position),
}

#[derive(Debug)]
pub enum AiState {
    Idling,
    Afraid,
    Angry,
}

impl Default for AiState {
    fn default() -> Self {
        AiState::Idling
    }
}

#[derive(Debug, Default)]
pub struct Ai {
    pub curr_state: AiState,
    // pub next_action: Action,
}

impl Ai {
    fn find_position_relative_to_player(
        &self,
        my_position: &Position,
        player_position: &Position,
        invert_angle: bool,
        distance: Option<f64>,
    ) -> Position {
        let distance = distance.unwrap_or(10.0);
        let angle = my_position.angle(player_position, None);
        let pos = my_position
            .go_distance_theta(distance, if invert_angle { 180.0 - angle } else { angle });
        pos
    }

    pub fn get_next_action(
        &mut self,
        player_pos: &Position,
        my_position: &Position,
        my_health: &Health,
        my_vision: &Vision,
    ) -> Action {
        match self.curr_state {
            AiState::Idling => {
                if my_vision.can_see(my_position, player_pos) {
                    self.curr_state = AiState::Angry;
                    Action::GoTo(player_pos.clone())
                } else {
                    Action::Wait
                }
            }
            AiState::Afraid => Action::GoTo(self.find_position_relative_to_player(
                player_pos,
                my_position,
                true,
                None,
            )),
            AiState::Angry => {
                if my_health.get_ratio() < 0.25 {
                    self.curr_state = AiState::Afraid;
                    Action::GoTo(self.find_position_relative_to_player(
                        player_pos,
                        my_position,
                        true,
                        None,
                    ))
                } else if my_position.distance_squared(player_pos) <= 1.0 {
                    Action::Attack(player_pos.clone())
                } else {
                    Action::GoTo(player_pos.clone())
                }
            }
        }
    }
}

mod tests {
    use super::*;

    #[test]
    fn test_vision() {
        let vision = Vision::new(2);
        let one = Position::new(10, 10);
        
        let two = Position::new(10, 10);
        assert!(vision.can_see(&one, &two));
        assert!(vision.can_see(&two, &one));

        let two = Position::new(9, 10);
        assert!(vision.can_see(&one, &two));
        assert!(vision.can_see(&two, &one));

        let two = Position::new(9, 9);
        // Distance should be sqrt(2) so that's still within vision range.
        assert!(vision.can_see(&one, &two));
        assert!(vision.can_see(&two, &one));

        let two = Position::new(10, 8);
        assert!(vision.can_see(&one, &two));
        assert!(vision.can_see(&two, &one));

        let two = Position::new(9, 8);
        // Distance should be sqrt(5) so shouldn't be within vision range.
        assert!(!vision.can_see(&one, &two));
        assert!(!vision.can_see(&two, &one));
    }
}
