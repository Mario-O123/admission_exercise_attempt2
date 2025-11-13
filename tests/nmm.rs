// Do not edit anything in this file. It is used for grading.
// The checksum of this file will be checked before running.

use std::fs;

use rust_nmm::nmm::{Action, Game, NmmGame, Piece, Player};

fn load_game(num: usize) -> Vec<Action> {
    let path = format!("tests/example_games/game{num}.txt");
    let content =
        fs::read_to_string(&path).unwrap_or_else(|_| panic!("Could not read file: {}", &path));
    let lines: Vec<&str> = content.lines().collect();
    lines
        .iter()
        .map(|a| a.parse().expect("parse failed"))
        .collect()
}

#[test]
fn test_board_new_is_empty() {
    let game = Game::new();
    assert_eq!(game.points().len(), 24);
    for point in game.points() {
        assert!(point.is_none());
    }
}

#[test]
fn test_basic_placement_and_win_condition() {
    let mut game = Game::new();
    let actions = ["W P 0", "B P 1", "W P 6", "B P 2", "W P 7", "W R 2"];
    for action in actions {
        let action: Action = action.parse().expect("parse failed");
        assert!(game.action(action).is_ok());
    }
    assert_eq!(game.winner(), None);
    let points = game.points();
    assert_eq!(points[0], Some(Piece::White));
    assert_eq!(points[1], Some(Piece::Black));
    assert_eq!(points[2], None); // removed
    assert_eq!(points[6], Some(Piece::White));
    assert_eq!(points[7], Some(Piece::White));
}

#[test]
fn test_illegal_moves() {
    let mut game = Game::new();
    let action1: Action = "W M 0 1".parse().expect("parse failed");
    assert!(game.action(action1).is_err()); // Cannot move in placing phase
    let action2: Action = "W P 24".parse().expect("parse failed");
    assert!(game.action(action2).is_err()); // Out of bounds
    let action3: Action = "W R 24".parse().expect("parse failed");
    assert!(game.action(action3).is_err()); // Out of bounds
}

#[test]
fn test_remove_only_non_mill_if_possible() {
    let mut game = Game::new();
    let actions = [
        "W P 0", "B P 3", "W P 6", "B P 4", "W P 7", // White forms mill 0-6-7
        "W R 4", // Remove Black's piece not in mill
        "B P 5", "W P 15", "B P 13", "W P 23", // White forms mill 7-15-23
        "W R 5",  // Remove Black's piece not in mill
        "B P 14", "W P 1", "B P 12", // Black forms mill 12-13-14
    ];
    for action in actions {
        let action: Action = action.parse().expect("parse failed");
        assert!(game.action(action).is_ok())
    }
    // Try to remove a piece in a mill when non-mill pieces exist
    let illegal_remove: Action = "B R 13".parse().expect("parse failed");
    assert!(game.action(illegal_remove).is_err());
}

#[test]
fn test_place_on_occupied_spot() {
    let mut game = Game::new();
    let actions: Vec<Action> = ["W P 0", "B P 1", "W P 0"]
        .iter()
        .map(|a| a.parse().expect("parse failed"))
        .collect();
    assert!(game.action(actions[0]).is_ok());
    assert!(game.action(actions[1]).is_ok());
    assert!(game.action(actions[2]).is_err())
}

#[test]
fn test_remove_own_piece() {
    let mut game = Game::new();
    let actions = ["W P 0", "B P 1", "W P 6", "B P 2", "W P 7"];
    for action in actions {
        let action: Action = action.parse().expect("parse failed");
        assert!(game.action(action).is_ok());
    }
    let illegal_remove: Action = "W R 0".parse().expect("parse failed");
    assert!(game.action(illegal_remove).is_err()); // Can't remove own piece
}

#[test]
fn test_undo() {
    let mut game = Game::new();
    let actions = ["W P 0", "B P 1", "W P 6"];
    for action in actions {
        let action: Action = action.parse().expect("parse failed");
        assert!(game.action(action).is_ok());
    }
    assert_eq!(game.points()[0], Some(Piece::White));
    assert!(game.undo().is_ok());
    assert_eq!(game.points()[6], None);
    assert!(game.undo().is_ok());
    assert_eq!(game.points()[1], None);
    assert!(game.undo().is_ok());
    assert_eq!(game.points()[0], None);
    assert!(game.undo().is_err()); // No more actions to undo
}

#[test]
fn test_undo_after_removal() {
    let mut game = Game::new();
    let actions = ["W P 0", "B P 1", "W P 6", "B P 2", "W P 7", "W R 2"];
    for action in actions {
        let action: Action = action.parse().expect("parse failed");
        game.action(action).expect("illegal action");
    }
    assert!(game.undo().is_ok());
    let points = game.points();
    assert_eq!(points[2], Some(Piece::Black));
}

#[test]
fn test_play_game1() {
    let mut game = Game::new();
    for action in load_game(1) {
        assert!(game.action(action).is_ok());
    }
    assert_eq!(game.winner(), Some(Player::White));
}

#[test]
fn test_play_game2() {
    let mut game = Game::new();
    for action in load_game(2) {
        assert!(game.action(action).is_ok());
    }
    assert_eq!(game.winner(), Some(Player::White));
}

#[test]
fn test_play_game3() {
    let mut game = Game::new();
    for action in load_game(3) {
        assert!(game.action(action).is_ok());
    }
    assert_eq!(game.winner(), Some(Player::White));
}

#[test]
fn test_play_game4() {
    let mut game = Game::new();
    for action in load_game(4) {
        assert!(game.action(action).is_ok());
    }
    assert_eq!(game.winner(), Some(Player::Black));
}

#[test]
fn test_play_game1_undo() {
    let mut game = Game::new();
    let actions = load_game(1);
    for i in 0..actions.len() {
        for _ in 0..i {
            assert!(game.undo().is_ok());
            assert_eq!(game.winner(), None);
        }
        assert_eq!(game.winner(), None);
        for action in actions.iter().take(i + 1) {
            assert!(game.action(*action).is_ok());
        }
    }
    assert_eq!(game.winner(), Some(Player::White));
}
