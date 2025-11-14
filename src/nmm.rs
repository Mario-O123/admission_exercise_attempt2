// Board representation, game logic, and public API for Nine Men's Morris
// This module will be tested from the ./tests folder
// Rules (incl. 'flying'): https://en.wikipedia.org/wiki/Nine_men%27s_morris
// White begins

use std::{fmt::Display, str::FromStr};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Color {
    Black,
    White,
}

impl Color {
    pub fn opposite(self) -> Color {
        match self {
            Color::Black => Color::White,
            Color::White => Color::Black,
        }
    }
}

pub type Player = Color;
pub type Piece = Color;
/// The board is represented by 24 points, numbered as follows:
/// 0––––––––1 –––––––2
/// |  8–––––9 ––––10 |
/// |  |  16–17–18 |  |
/// 7 –15–23    19–11–3
/// |  |  22–21–20 |  |
/// |  14––––13––––12 |
/// 6––––––––5 –––––––4
pub type Point = usize; // 0–23

const BOARD_POINTS: usize = 24;

const NEIGHBORS: [&[usize]; 24] = [
    &[1, 7],
    &[0, 2, 9],
    &[1, 3],
    &[2, 4, 11],
    &[3, 5],
    &[4, 6, 13],
    &[5, 7],
    &[0, 6, 15],
    &[9, 15],
    &[1, 8, 10, 17],
    &[9, 11],
    &[3, 10, 12, 19],
    &[11, 13],
    &[5, 12, 14, 21],
    &[13, 15],
    &[7, 8, 14, 23],
    &[17, 23],
    &[9, 16, 18],
    &[17, 19],
    &[11, 18, 20],
    &[19, 21],
    &[13, 20, 22],
    &[21, 23],
    &[15, 16, 22],
];

const MILLS: [[usize; 3]; 16] = [
    [0, 1, 2],
    [2, 3, 4],
    [4, 5, 6],
    [0, 6, 7],
    [8, 9, 10],
    [10, 11, 12],
    [12, 13, 14],
    [8, 14, 15],
    [16, 17, 18],
    [18, 19, 20],
    [20, 21, 22],
    [16, 22, 23],
    [1, 9, 17],
    [3, 11, 19],
    [5, 13, 21],
    [7, 15, 23],
];

/// Describes the contents of an action.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ActionKind {
    Place(Point),
    Move(Point, Point),
    Remove(Point),
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Action {
    pub player: Player,
    pub action: ActionKind,
}

// This implementation is used extensively for testing
impl FromStr for Action {
    type Err = &'static str;

    /// Example inputs:
    /// "W P 0" - White places at 0
    /// "B M 0 1" - Black moves from 0 to 1
    /// "W R 5" - White removes at 5
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split_whitespace().collect();
        if parts.len() < 3 {
            return Err("Invalid action format");
        }
        let player = match parts[0] {
            "W" => Player::White,
            "B" => Player::Black,
            _ => return Err("Invalid player"),
        };
        let action = match parts[1] {
            "P" => {
                let point: Point = parts[2].parse().map_err(|_| "Invalid point")?;
                ActionKind::Place(point)
            }
            "M" => {
                if parts.len() != 4 {
                    return Err("Invalid move format");
                }
                let from: Point = parts[2].parse().map_err(|_| "Invalid from point")?;
                let to: Point = parts[3].parse().map_err(|_| "Invalid to point")?;
                ActionKind::Move(from, to)
            }
            "R" => {
                let point: Point = parts[2].parse().map_err(|_| "Invalid point")?;
                ActionKind::Remove(point)
            }
            _ => return Err("Invalid action type"),
        };
        Ok(Action { player, action })
    }
}

impl Display for Action {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let player_str = match self.player {
            Player::White => "W",
            Player::Black => "B",
        };
        let action_str = match self.action {
            ActionKind::Place(p) => format!("P {p}"),
            ActionKind::Move(from, to) => format!("M {from} {to}"),
            ActionKind::Remove(p) => format!("R {p}"),
        };
        write!(f, "{player_str} {action_str}")
    }
}

pub trait NmmGame {
    /// Creates a new instance with an empty board.
    fn new() -> Self;
    /// Applies the given action.
    fn action(&mut self, action: Action) -> Result<(), &'static str>;
    /// Undoes the last action.
    /// This should fail if there is no last action to be undone.
    fn undo(&mut self) -> Result<(), &'static str>;
    /// All poinst of the game board
    fn points(&self) -> &[Option<Piece>; 24];
    /// Returns if there is currently a winner.
    /// There are two win-conditions:
    /// - one player has removed 7 pieces of the opponent
    /// - one player cannot make a legal move
    fn winner(&self) -> Option<Player>;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Phase {
    Placing,
    Moving,
    //Flying will be handled per player in Moving, when a player has 3 pieces left
    //having a Flying phase would require me to handel changing phases Moving and Flying if 1 player is in Flying while  the other isnt = annoying
}

#[derive(Clone, Debug)]
struct GameState {
    board: [Option<Piece>; BOARD_POINTS],
    current_player: Player,
    white_placed: u8,
    black_placed: u8,
    phase: Phase,
    pending_removal: bool,
}

/*
Complete the struct called `Game` that implements the `NmmGame` trait. All functionality exposed by
the trait should be implemented.
*/

pub struct Game {
    board: [Option<Piece>; BOARD_POINTS],
    current_player: Player,
    //these ..._placed don't change when removing pieces
    white_placed: u8,
    black_placed: u8,
    phase: Phase,
    pending_removal: bool, //this attribut is used to control that a player has to immediately remove after they formed a mill
    game_states: Vec<GameState>, //saves the states of a game
}

impl Game {
    fn save_state(&mut self) {
        self.game_states.push(GameState {
            board: self.board,
            current_player: self.current_player,
            white_placed: self.white_placed,
            black_placed: self.black_placed,
            phase: self.phase,
            pending_removal: self.pending_removal,
        });
    }    
    fn in_bounds(p: Point) -> bool {
        p < BOARD_POINTS
    }

    fn count_pieces(&self, player: Player) -> usize {
        self.board.iter().filter(|s| **s == Some(player)).count()
    }
    
    //forms_mill checks if after a player moved or placed a piece, if this action now made a mill, use it to set pending_removal true
    fn forms_mill(&self, point: Point, player: Player) -> bool {
        for mill in MILLS.iter() {
            if mill.contains(&point) && mill.iter().all(|&p| self.board[p] == Some(player)) {
                return true;
            }
        }
        false
    }
    //is_part_of_mill checs if a piece which is already on board part of a mill or not = needed if a remove action is pending
    fn is_part_of_mill(&self, point: Point) -> bool {
        if let Some(color) = self.board[point] {
            for mill in MILLS.iter() {
                if mill.contains(&point) && mill.iter().all(|&p| self.board[p] == Some(color)) {
                    return true;
                }
            }
        }
        false
    }

    //task: check if the opp has 1 piece at minimum, which isnt in a mil so can be removed
    fn opponent_has_non_mill_piece(&self, opponent: Player) -> bool {
        self.board.iter().enumerate().any(|(i, s)| {
            *s == Some(opponent) && !self.is_part_of_mill(i)
        })
    }
    //task: if can_player_fly is true, then the player can move to any free point on the board
    fn can_player_fly(&self, player: Player) -> bool {
        self.count_pieces(player) == 3
    }

    //task: a player can only win, if they have at minimum 1 legal move they can make
    fn has_legal_move(&self, player: Player) -> bool {
        //during Placing, dont count a blockade as no legal moves left
        if matches!(self.phase, Phase::Placing) {
            return true;
        }
        let player_count = self.count_pieces(player);
        if player_count == 0 {
            return false;
        }
        
        //if a player can fly, then only one of their pieces and 1 free point is needed to make legal move
        if self.can_player_fly(player) {
            return self.board.iter().any(|s| s.is_none());
        }
        //during Moving, the player has to use their own piece and has to be able to move to a neighbor
        for (i, slot) in self.board.iter().enumerate() {
            if *slot == Some(player) && NEIGHBORS[i].iter().any(|&n| self.board[n].is_none()) {
                return true;
            }
        }
        false
    }

    fn maybe_update_phase_after_action(&mut self) {
        //Wont treat Flying as own phase, but use still Moving but allow flying when a player has =3 pieces

        //here, swithc from Placing to Moving after 18 pieces have been placed
        if matches!(self.phase, Phase::Placing) && (self.white_placed + self.black_placed) == 18 {
            //cahnge the phase to Moving, if pending_removal is false = a remove action doesn't have to happen immediately
            if !self.pending_removal {
                self.phase = Phase::Moving;
            }
        }
    }

    fn switch_turn(&mut self) {
        self.current_player = self.current_player.opposite();
    }
}



impl NmmGame for Game {
    fn new() -> Self {
        Game {
            board: [None; BOARD_POINTS],
            current_player: Player::White,
            white_placed: 0,
            black_placed: 0,
            phase: Phase::Placing,
            pending_removal: false,
            game_states: Vec::new(),
        }
    }

    fn action(&mut self, action: Action) -> Result<(), &'static str> {
        // check if the right player is thaking the action
        if action.player != self.current_player {
            return Err("Not this player's turn");
        }

        match action.action {
            ActionKind::Place(p) => {
                //Allow placing, only if pending_removal is false AND we are in Placing
                if self.pending_removal {
                    return Err("Removal required");
                }
                if !matches!(self.phase, Phase::Placing) {
                    return Err("Cannot place outside placing phase");
                }
                if !Self::in_bounds(p) {
                    return Err("Out of bounds");
                }
                if self.board[p].is_some() {
                    return Err("Point occupied");
                }

                //before doing the action, save the state of the current game
                self.save_state();

                //here, the actual Placing action
                self.board[p] = Some(self.current_player);
                match self.current_player {
                    Player::White => self.white_placed += 1,
                    Player::Black => self.black_placed += 1,
                }

                //was a mill formed after the action?
                if self.forms_mill(p, self.current_player) {
                    self.pending_removal = true; //no player switch cause there has to be a remove action first
                } else {
                    self.switch_turn();
                }

                //change phase, if 18 pieces placed and no pending_removal
                self.maybe_update_phase_after_action();
                Ok(())
            }

            ActionKind::Move(from, to) => {
                if self.pending_removal {
                    return Err("Removal required");
                }
                if !Self::in_bounds(from) || !Self::in_bounds(to) {
                    return Err("Out of bounds");
                }
                if from == to {
                    return Err("Invalid move");
                }
                if self.board[from] != Some(self.current_player) {
                    return Err("Source not owned by player");
                }
                if self.board[to].is_some() {
                    return Err("Destination occupied");
                }
                if matches!(self.phase, Phase::Placing) {
                    return Err("Cannot move in placing phase");
                }

                //player can fly in Moving if they have 3 pieces left
                let can_fly = self.can_player_fly(self.current_player);
                if !can_fly {
                    //if the player isnt allowed to fly, they have to move to a neigbor
                    if !NEIGHBORS[from].contains(&to) {
                        return Err("Not a neighbor");
                    }
                }

                //save state before actual action
                self.save_state();

                //actual Move
                self.board[from] = None;
                self.board[to] = Some(self.current_player);

                //was mill formed after the move was made?
                if self.forms_mill(to, self.current_player) {
                    self.pending_removal = true;
            
                } else {
                    self.switch_turn();
                }

                Ok(())
            }

            ActionKind::Remove(p) => {
                //use pending_removal to check if a remove is allowed
                if !self.pending_removal {
                    return Err("No removal pending");
                }
                if !Self::in_bounds(p) {
                    return Err("Out of bounds");
                }
                let victim = self.board[p].ok_or("No piece to remove")?;
                if victim == self.current_player {
                    return Err("Cannot remove own piece");
                }

                //if opp has pieces which are not in a mill, they cant be removed
                let opponent = self.current_player.opposite();
                let opp_has_non_mill = self.opponent_has_non_mill_piece(opponent);
                if opp_has_non_mill && self.is_part_of_mill(p) {
                    return Err("Must remove non mill piece if possible");
                }

                //save state before action
                self.save_state();

                //actual removal
                self.board[p] = None;
                self.pending_removal = false;

                //if all 18 pieces are set for Placing, then after removal change phase
                self.maybe_update_phase_after_action();

                self.switch_turn();

                Ok(())
            }
        }
    }

    fn undo(&mut self) -> Result<(), &'static str> {
        match self.game_states.pop() {
            Some(prev) => {
                self.board = prev.board;
                self.current_player = prev.current_player;
                self.white_placed = prev.white_placed;
                self.black_placed = prev.black_placed;
                self.phase = prev.phase;
                self.pending_removal = prev.pending_removal;
                Ok(())
            }
            None => Err("No more actions to undo"),
        }
    }

    fn points(&self) -> &[Option<Piece>; 24] {
        &self.board
    }

    fn winner(&self) -> Option<Player> {
        //Never announce a winner in Placin, cause not all 18 pieces for Placing have been set
    if matches!(self.phase, Phase::Placing)|| self.pending_removal {
        return None;
    }

    //if a player has less than 3 pieces, they have lost
    let white_pieces = self.count_pieces(Player::White);
    let black_pieces = self.count_pieces(Player::Black);
    if white_pieces < 3 {
        return Some(Player::Black);
    }
    if black_pieces < 3 {
        return Some(Player::White);
    }

    //if a player cant fly and is in Moving, and cannot make any legal moves, they have lost
    if white_pieces > 3 && !self.has_legal_move(Player::White) {
        return Some(Player::Black);
    }
    if black_pieces > 3 && !self.has_legal_move(Player::Black) {
        return Some(Player::White);
    }

    None
    }
}


// For grading this assignment, the tests in the `tests` folder will be used.
// Small unit tests are generally included in the same file as the code they test.
// You are free to add more tests here if you wish.
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_board_new_is_empty() {
        let game = Game::new();
        for pos in *game.points() {
            assert_eq!(pos, None);
        }
    }
    #[test]
    fn test_board_new_is_empty_local() {
        let game = Game::new();
        assert_eq!(game.points().len(), 24);
        assert!(game.points().iter().all(|p| p.is_none()));
        assert_eq!(game.current_player, Player::White);
        assert!(matches!(game.phase, Phase::Placing));
        assert!(!game.pending_removal);
    }
    //testing count_pieces
    #[test]
    fn test_count_pieces() {
    let mut game = Game::new();
    assert_eq!(game.count_pieces(Player::White), 0);
    assert_eq!(game.count_pieces(Player::Black), 0);

    game.board[0] = Some(Player::White);
    game.board[1] = Some(Player::White);
    game.board[2] = Some(Player::Black);
    assert_eq!(game.count_pieces(Player::White), 2);
    assert_eq!(game.count_pieces(Player::Black), 1);

    //check case: count after 1 less White piece
    game.board[1] = None;
    assert_eq!(game.count_pieces(Player::White), 1);
    }
    //testing in_bounds
    #[test]
    fn test_in_bounds() {
    assert!(Game::in_bounds(0));
    assert!(Game::in_bounds(23));
    assert!(!Game::in_bounds(24));
    }

    //testing switch_turns
    #[test]
    fn test_switch_turn() {
    let mut game = Game::new();

    //check case: by default current_player is White, so after switch, it should be Black
    game.switch_turn();
    assert_eq!(game.current_player, Player::Black);

    //check case: another switch will cause White to be current_player
    game.switch_turn();
    assert_eq!(game.current_player, Player::White);
    }

    //testing forms_mill
    #[test]
    fn test_place_and_form_mill() {
    let mut game = Game::new();

    //setup: White forms mill 0-6-7
    game.board[0] = Some(Player::White);
    game.board[6] = Some(Player::White);
    game.board[7] = Some(Player::White);

    assert!(game.forms_mill(7, Player::White));
    assert!(game.forms_mill(0, Player::White));
    assert!(game.forms_mill(6, Player::White));

    //check case: Black doesnt form a mill
    assert!(!game.forms_mill(7, Player::Black));

    //check case: cannot form a mill with 2 pieces
    game.board[6] = None;
    assert!(!game.forms_mill(7, Player::White));
    }

    //testing: is_part_of_mill
    #[test]
    fn test_is_part_of_mill() {
    let mut game = Game::new();

    game.board[0] = Some(Player::White);
    game.board[6] = Some(Player::White);
    game.board[7] = Some(Player::White);

    assert!(game.is_part_of_mill(0));
    assert!(game.is_part_of_mill(6));
    assert!(game.is_part_of_mill(7));

    //check case: en empty point shouldnt be part of a mill
    assert!(!game.is_part_of_mill(1));

    //check case: Black having a single piece doesnt form a mill
    game.board[3] = Some(Player::Black);
    assert!(!game.is_part_of_mill(3));

    //check case: after a piece from a White mill moved, the other pieces in that former mill arent in a mill anymore
    game.board[6] = None;
    assert!(!game.is_part_of_mill(0));
    assert!(!game.is_part_of_mill(7));
    }

    #[test]
    fn test_is_part_of_mill_multiple_mills() {
    let mut game = Game::new();

    //check case: create first mill
    game.board[0] = Some(Player::White);
    game.board[1] = Some(Player::White);
    game.board[2] = Some(Player::White);

    assert!(game.is_part_of_mill(1));
    assert!(game.is_part_of_mill(0));
    assert!(game.is_part_of_mill(2));

    //check case: break first mill
    game.board[0] = None;
    game.board[2] = None;
    assert!(!game.is_part_of_mill(1));

    //check case: build new mill
    game.board[9] = Some(Player::White);
    game.board[17] = Some(Player::White);
    assert!(game.is_part_of_mill(1));
    assert!(game.is_part_of_mill(9));
    assert!(game.is_part_of_mill(17));
    }

    //testing: can_player_fly
    //note: dont have to set phase: Moving, because we never
    //call can_player_fly in the Placing section of action()
    #[test]
    fn test_can_player_fly() {
    let mut game = Game::new();

    //check case: if White has 3 pieces, it can fly
    game.board[0] = Some(Player::White);
    game.board[1] = Some(Player::White);
    game.board[2] = Some(Player::White);
    assert!(game.can_player_fly(Player::White));

    //check case: if White has > 3 pieces it cannot fly
    game.board[3] = Some(Player::White);
    assert!(!game.can_player_fly(Player::White));

    //check again: White cannot fly with =/= 3 pieces
    game.board[3] = None;
    game.board[2] = None;
    assert!(!game.can_player_fly(Player::White));

    //check case: Black has no pieces so cannot fly
    assert!(!game.can_player_fly(Player::Black));
    }

    //tesing: has_legal_move
    #[test]
    fn test_has_legal_move() {
    let mut game = Game::new();

    //setup: in Moving because in Placing the players have always legal placing
    game.phase = Phase::Moving;

    //check case: White has piece on 0 and can move to a free neighbor (1 or 7)
    game.board[0] = Some(Player::White);
    assert!(game.has_legal_move(Player::White));
 
    //check case: blocked player White has no legal move because Black blocked them with 1, 7
    game.board[1] = Some(Player::Black);
    game.board[7] = Some(Player::Black);
    assert!(!game.has_legal_move(Player::White));

    //check case: White has legal move because they can fly (3 pieces) and choose 1 free point
    game.board = [None; 24];
    game.board[0] = Some(Player::White);
    game.board[6] = Some(Player::White);
    game.board[7] = Some(Player::White);
    assert!(game.has_legal_move(Player::White));

    //check case: player cannot make a legal move, if they have no pieces in Moving Phase, they have already lost
    game.board = [None; 24];
    assert!(!game.has_legal_move(Player::White));

    //check case: player can always make a legal move (move as in placing)
    game.phase = Phase::Placing;
    assert!(game.has_legal_move(Player::White));
    }

    //testing of opponent_has_non_mill_piece
    #[test]
    fn test_opponent_has_non_mill_piece() {
    let mut game = Game::new();

    //setup: White is current_player by default, so opp is Black
    let opponent = Player::Black;

    //check case: opp Black has no pieces on board, so fn should fail
    assert!(!game.opponent_has_non_mill_piece(opponent));

    //check case: opp Black has 1 piece that is part of a mill, so fn should fail
    game.board[0] = Some(Player::Black);
    game.board[1] = Some(Player::Black);
    game.board[2] = Some(Player::Black);
    assert!(!game.opponent_has_non_mill_piece(opponent));

    //check case: opp Black has 1 piece which is not in a mill, so fn should be true
    game.board = [None; 24];
    game.board[0] = Some(Player::Black);
    assert!(game.opponent_has_non_mill_piece(opponent));

    //check case: opp Black has multiple pieces but only 1 is not in mill (9)
    game.board = [None; 24];
    game.board[0] = Some(Player::Black);
    game.board[1] = Some(Player::Black);
    game.board[2] = Some(Player::Black);
    game.board[9] = Some(Player::Black);
    assert!(game.opponent_has_non_mill_piece(opponent));
    }

    //testing: maybe_update_phase_after_action
    #[test]
    fn test_maybe_update_phase_after_action() {
    let mut game = Game::new();

    //setup: start in Placing
    assert!(matches!(game.phase, Phase::Placing));

    //check case: if not all 18 pieces are placed, no change of phase
    game.white_placed = 5;
    game.black_placed = 5;
    game.maybe_update_phase_after_action();
    assert!(matches!(game.phase, Phase::Placing));

    //check case: there are 18 pieces placed, but pending_removal is true, no player switch, we are in Placing until remove happens
    game.white_placed = 9;
    game.black_placed = 9;
    game.pending_removal = true;
    game.maybe_update_phase_after_action();
    assert!(matches!(game.phase, Phase::Placing));

    //check case: 18 pieces placed, but pending_removal is false, so weitch to Moving
    game.pending_removal = false;
    game.maybe_update_phase_after_action();
    assert!(matches!(game.phase, Phase::Moving));
    }

    //testing action

    #[test]
    fn test_action_place_wrong_turn() {
    let mut game = Game::new();

    //check case: White starts, but Black tries to place
    let a: Action = "B P 0".parse().unwrap();
    assert!(game.action(a).is_err());
    }

    #[test]
    fn test_action_place_out_of_bounds() {
    let mut game = Game::new();

    let a: Action = "W P 24".parse().unwrap();
    assert!(game.action(a).is_err());
    }

    #[test]
    fn test_action_place_on_occupied() {
    let mut game = Game::new();

    let a1: Action = "W P 0".parse().unwrap();
    let a2: Action = "B P 0".parse().unwrap();

    assert!(game.action(a1).is_ok());
    assert!(game.action(a2).is_err());
    }

    #[test]
    fn test_action_place_during_pending_removal() {
    let mut game = Game::new();
    //setup: White forms a mill 0-6-7

    for act in ["W P 0", "B P 1", "W P 6", "B P 2", "W P 7"] {
        game.action(act.parse().unwrap()).unwrap();
    }

    //check case: cause White placed on 7 last and formed mill, pending_removal is true
    let illegal: Action = "W P 10".parse().unwrap();
    assert!(game.action(illegal).is_err());
    }

    #[test]
    fn test_action_move_in_placing_phase_fail() {
    let mut game = Game::new();

    let move_action: Action = "W M 0 1".parse().unwrap();
    assert!(game.action(move_action).is_err());
    }

    #[test]
    fn test_action_move_to_occupied() {
    let mut game = Game::new();
    game.board[0] = Some(Player::White);
    game.board[1] = Some(Player::Black);

    game.phase = Phase::Moving;

    let move_action: Action = "W M 0 1".parse().unwrap();
    assert!(game.action(move_action).is_err());
    }

    #[test]
    fn test_action_move_not_neighbor_when_not_flying() {
    let mut game = Game::new();
    game.phase = Phase::Moving;

    //setup: White has more than 3 pieces, so no flying
    game.board[0] = Some(Player::White);
    game.board[6] = Some(Player::White);
    game.board[7] = Some(Player::White);
    game.board[10] = Some(Player::White);

    //check case: White trying to move to non neighbor should fail
    let move_action: Action = "W M 0 10".parse().unwrap();
    assert!(game.action(move_action).is_err());
    }

    #[test]
    fn test_action_move_flying_allowed() {
    let mut game = Game::new();
    game.phase = Phase::Moving;

    //White has exactly 3 pieces, so flying is allowed 
    game.board[0] = Some(Player::White);
    game.board[6] = Some(Player::White);
    game.board[7] = Some(Player::White);

    //check case: moving to any point should be allowed
    let move_action: Action = "W M 0 10".parse().unwrap();
    assert!(game.action(move_action).is_ok());
    }

    #[test]
    fn test_action_remove_without_pending() {
    let mut game = Game::new();

    let r: Action = "W R 0".parse().unwrap();
    assert!(game.action(r).is_err());
    }

    #[test]
    fn test_action_remove_out_of_bounds() {
    let mut game = Game::new();

    let remove_action: Action = "W R 24".parse().unwrap();
    assert!(game.action(remove_action).is_err());
    }


    #[test]
    fn test_action_remove_own_piece_in_pending() {
    let mut game = Game::new();

    //setup: its White's turn and has to remove a piece, but tries to remove own piece
    game.pending_removal = true;

    game.board[3] = Some(Player::White); // own piece
    game.current_player = Player::White;

    let remove_action: Action = "W R 3".parse().unwrap();
    assert!(game.action(remove_action).is_err());
    }

    #[test]
    fn test_action_remove_mill_piece_when_non_mill_exists() {
    let mut game = Game::new();

    //setup: Black has a mill 0-1-2 but also 1 non mill at 5
    //and White has to remove now
    game.pending_removal = true;
    game.current_player = Player::White;

    game.board[0] = Some(Player::Black);
    game.board[1] = Some(Player::Black);
    game.board[2] = Some(Player::Black);
    game.board[5] = Some(Player::Black);

    //check case: White removing a Black piece in mill (0) should fail
    let remove_action: Action = "W R 0".parse().unwrap();
    assert!(game.action(remove_action).is_err());
    }

    #[test]
    fn test_action_full_sequence() {
    let mut game = Game::new();

    let seq = [
        "W P 0",
        "B P 3",
        "W P 6",
        "B P 4",
        "W P 7", //at this point White forms mill 0-6-7, has to remove a piece
        "W R 4",
        "B P 5",
        "W P 15",
        "B P 13",
        "W P 23", //at this point White forms a mill again 15-23-7, so pending_removal is true, no player switch
    ];

    for a in seq {
        assert!(game.action(a.parse().unwrap()).is_ok());
    }

    //check case: did White remove after first mill adn does White have to remove again?
    assert_eq!(game.board[4], None);    // removed
    assert_eq!(game.current_player, Player::White);
    }

    //testing of undo
    #[test]
    fn test_undo_after_simple_place() {
    let mut game = Game::new();

    let action: Action = "W P 0".parse().unwrap();
    game.action(action).unwrap();

    assert_eq!(game.board[0], Some(Player::White));
    assert_eq!(game.current_player, Player::Black);

    game.undo().unwrap();

    assert_eq!(game.board[0], None);
    assert_eq!(game.current_player, Player::White);
    assert_eq!(game.white_placed, 0);
    assert_eq!(game.phase, Phase::Placing);
    }

    #[test]
    fn test_undo_chain_place() {
    let mut game = Game::new();

    let sequence = ["W P 0", "B P 1", "W P 6"];

    for s in sequence {
        game.action(s.parse().unwrap()).unwrap();
    }
    assert_eq!(game.board[6], Some(Player::White));

    //check case: undo removes the last action (W P 6)
    game.undo().unwrap();
    assert_eq!(game.board[6], None);
    assert_eq!(game.current_player, Player::White);

    //check cace: repeated undo removes action (B P 1)
    game.undo().unwrap();
    assert_eq!(game.board[1], None);
    assert_eq!(game.current_player, Player::Black);

    //check case: repeated undo removes action (W P 0)
    game.undo().unwrap();
    assert_eq!(game.board[0], None);
    assert_eq!(game.current_player, Player::White);

    //check case: now that all actions got undone, a repeated undo() should fail
    assert!(game.undo().is_err());
    }

    #[test]
    fn test_undo_after_move() {
    let mut game = Game::new();

    //setup: undo for Moving phase
    game.board[0] = Some(Player::White);
    game.board[1] = None;
    game.white_placed = 9;
    game.black_placed = 9;
    game.phase = Phase::Moving;

    let move_action: Action = "W M 0 1".parse().unwrap();
    game.action(move_action).unwrap();

    assert_eq!(game.board[0], None);
    assert_eq!(game.board[1], Some(Player::White));
    assert_eq!(game.current_player, Player::Black);

    game.undo().unwrap();

    assert_eq!(game.board[0], Some(Player::White));
    assert_eq!(game.board[1], None);
    assert_eq!(game.current_player, Player::White);
    }  

    #[test]
    fn test_undo_after_remove() {
    let mut game = Game::new();

    //setup: sequence of actions leads to white forming mill at 0-6-7
    for s in ["W P 0", "B P 1", "W P 6", "B P 2", "W P 7"] {
        game.action(s.parse().unwrap()).unwrap();
    }

    //now that White formed mill, pending_removal should be true
    assert!(game.pending_removal);

    let remove_action: Action = "W R 1".parse().unwrap();
    game.action(remove_action).unwrap();

    //check case: after remove action happens, 1 is empty and its Black's turn
    assert_eq!(game.board[1], None);
    assert_eq!(game.current_player, Player::Black);

    game.undo().unwrap();
    //check case: after undo(), Black has piece at 1 again, but White still has to remove immediately so it's White's turn
    assert_eq!(game.board[1], Some(Player::Black));
    assert!(game.pending_removal);
    assert_eq!(game.current_player, Player::White);
    }

    #[test]
    fn test_undo_after_phase_switch() {
    let mut game = Game::new();

    //setup: just barely before ending Placing phase, 1 placing needed
    game.white_placed = 8;
    game.black_placed = 9;
    game.phase = Phase::Placing;

    //this action should cause phase switch, if no pending_removal after placing
    let action = "W P 5".parse().unwrap();
    game.action(action).unwrap();

    //check case: phase adjusted after placing
    assert!(matches!(game.phase, Phase::Moving));

    //check case: undo should restore Placing phase
    game.undo().unwrap();
    assert!(matches!(game.phase, Phase::Placing));
    }

    //tests for winner()

    #[test]
    fn test_no_winner_during_placing() {
    let mut game = Game::new();

    //check case: winner always None doring Placing
    assert!(matches!(game.phase, Phase::Placing));
    assert_eq!(game.winner(), None);

    //check case: winner still None after White placed at 0
    game.action("W P 0".parse().unwrap()).unwrap();
    assert_eq!(game.winner(), None);

    //check case: winner still None after Black placed at 3
    game.action("B P 3".parse().unwrap()).unwrap();
    assert_eq!(game.winner(), None);

    //setup: White forms a mill and has to remove afterward
    game.action("W P 6".parse().unwrap()).unwrap();
    game.action("B P 4".parse().unwrap()).unwrap();
    game.action("W P 7".parse().unwrap()).unwrap();
    assert!(game.pending_removal);

    //check case: no winner after White removes piece
    game.action("W R 4".parse().unwrap()).unwrap();
    assert_eq!(game.winner(), None);

    assert_eq!(game.winner(), None);
    }


    #[test]
    fn test_winner_under_three() {
    let mut game = Game::new();

    //setup: White has <3 pieces in Moving and Black has >= 3
    game.phase = Phase::Moving;

    game.board = [None; 24];
    game.board[0] = Some(Player::White);
    game.board[1] = Some(Player::White);

    game.board[5] = Some(Player::Black);
    game.board[6] = Some(Player::Black);
    game.board[7] = Some(Player::Black);

    assert_eq!(game.winner(), Some(Player::Black));
    }

    #[test]
    fn test_winner_by_blockade() {
    let mut game = Game::new();
    game.phase = Phase::Moving;

    //setup: White has 4 pieces, but all neighbors are blocked
    game.board[0] = Some(Player::White);
    game.board[6] = Some(Player::White);
    game.board[5] = Some(Player::White);
    game.board[4] = Some(Player::White);

    game.board[1] = Some(Player::Black); // neighbor of 0
    game.board[7] = Some(Player::Black);  // neigbor of 0 & 6
    game.board[3] = Some(Player::Black);  // neighbor of 4
    game.board[13] = Some(Player::Black); // neighbor of 5

    //check case: White should have no legal moves left
    assert!(!game.has_legal_move(Player::White));

    //check case: because White is blocked, Black wins
    assert_eq!(game.winner(), Some(Player::Black));
    }

    #[test]
    fn test_no_winner_when_player_can_fly() {
    let mut game = Game::new();
    game.phase = Phase::Moving;

    //setup: White has 3 stones (so can fly) and is blocked
    game.board[0] = Some(Player::White);
    game.board[6] = Some(Player::White);
    game.board[7] = Some(Player::White);

    game.board[1] = Some(Player::Black);
    game.board[5] = Some(Player::Black);
    game.board[15] = Some(Player::Black);

    //check case: Black cannot win by blocking White

    assert!(game.can_player_fly(Player::White));
    assert_eq!(game.winner(), None);
    }


    #[test]
    fn test_no_winner_during_pending_removal() {
    let mut game = Game::new();
    game.phase = Phase::Moving;

    //setup: White has 4 pieces, so cannot fly, Black blocks all neighbors of White pieces
    game.board[0] = Some(Player::White);
    game.board[6] = Some(Player::White);
    game.board[5] = Some(Player::White);
    game.board[4] = Some(Player::White);

    game.board[1] = Some(Player::Black);
    game.board[7] = Some(Player::Black);
    game.board[3] = Some(Player::Black);
    game.board[13] = Some(Player::Black);

    //check case: white would currently lose because all pieces are blocked, but before
    //checking winner, set pending_removal true
    assert!(!game.has_legal_move(Player::White));

    game.pending_removal = true;

    //check case: White is blocked but current_player and pending_removal is true, so Black hasnt won yet
    assert_eq!(game.winner(), None);
    }











































}