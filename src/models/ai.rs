use crate::models::stats::Health;
use crate::models::{DistanceMetric, Position};

#[derive(Debug)]
pub struct Vision {
    view_range: usize,
}

impl Vision {
    pub fn new(view_range: usize) -> Self {
        tracing::trace!(view_range = view_range, "Creating vision");
        Vision { view_range }
    }
    pub fn can_see(&self, self_pos: &Position, position: &Position) -> bool {
        let can_see = self_pos.distance(position, &DistanceMetric::EuclideanSquared)
            <= (self.view_range.pow(2) as f64);
        tracing::debug!(can_see = can_see, self_pos = ?self_pos, position = ?position);
        can_see
    }
}

#[derive(Debug, PartialEq)]
pub enum Action {
    GoTo(Position),
    Wait,
    Attack(Position),
}

#[derive(Debug, PartialEq)]
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
        let angle = my_position.angle(player_position);
        let pos = my_position
            .go_distance_theta(distance, if invert_angle { 180.0 - angle } else { angle });
        tracing::debug!("Found position relative to {player_position:?} to be {pos:?}");
        pos
    }

    pub fn get_next_action(
        &mut self,
        player_pos: &Position,
        my_position: &Position,
        my_health: &Health,
        my_vision: &Vision,
    ) -> Action {
        let action_to_take = match self.curr_state {
            AiState::Idling => {
                if my_vision.can_see(my_position, player_pos) {
                    self.curr_state = AiState::Angry;
                    Action::GoTo(player_pos.clone())
                } else {
                    Action::Wait
                }
            }
            AiState::Afraid => {
                if !my_vision.can_see(my_position, player_pos) {
                    self.curr_state = AiState::Idling;
                    Action::Wait
                } else {
                    Action::GoTo(self.find_position_relative_to_player(
                        player_pos,
                        my_position,
                        true,
                        None,
                    ))
                }
            }
            AiState::Angry => {
                if !my_vision.can_see(my_position, player_pos) {
                    self.curr_state = AiState::Idling;
                    Action::Wait
                } else if my_health.get_ratio() < 0.25 {
                    self.curr_state = AiState::Afraid;
                    Action::GoTo(self.find_position_relative_to_player(
                        player_pos,
                        my_position,
                        true,
                        None,
                    ))
                } else if my_position.distance_squared(player_pos) <= 2.0 {
                    // Allow AIs to reach the player if they're diagonally next to each other.
                    // (Have a Euclidean distance of sqrt(2))
                    Action::Attack(player_pos.clone())
                } else {
                    Action::GoTo(player_pos.clone())
                }
            }
        };
        tracing::trace!(
            "Given Player Pos {player_pos:?}, curr_state={:?}, my position={my_position:?}, my_health={my_health:?}, my_vision={my_vision:?} => action={action_to_take:?}",
            self.curr_state
        );
        action_to_take
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

        let vision = Vision::new(3);
        // Sqrt(5) is between 2 and 2.5 so we should be in vision range.
        assert!(vision.can_see(&one, &two));
        assert!(vision.can_see(&two, &one));
    }

    #[test]
    fn test_ai_get_next_action() {
        let player_position = Position::new(10, 10);
        let vision = Vision::new(2);
        let mut health = Health::new(10);
        let mut ai = Ai::default();
        let ai_pos = Position::new(0, 0);

        let action = ai.get_next_action(&player_position, &ai_pos, &health, &vision);
        assert_eq!(action, Action::Wait);
        assert_eq!(ai.curr_state, AiState::Idling);

        let ai_pos = Position::new(9, 9);
        let action = ai.get_next_action(&player_position, &ai_pos, &health, &vision);
        assert_eq!(action, Action::GoTo(player_position.clone()));
        assert_eq!(ai.curr_state, AiState::Angry);

        let action = ai.get_next_action(&player_position, &ai_pos, &health, &vision);
        assert_eq!(action, Action::Attack(player_position.clone()));
        assert_eq!(ai.curr_state, AiState::Angry);

        // We're now big hurt
        health.current_health = 1;
        let action = ai.get_next_action(&player_position, &ai_pos, &health, &vision);
        match action {
            Action::GoTo(pos) => {
                // Make sure it's not the same position as the player anymore.
                assert_ne!(pos, player_position);
                // Make sure the AI is going the completely opposite direction.
                let diff_angle = player_position.angle(&pos) - ai_pos.angle(&player_position);
                assert!(diff_angle - 180.0 < 1e-3);
            }
            _ => assert!(false),
        }
        assert_eq!(ai.curr_state, AiState::Afraid);
    }
}
