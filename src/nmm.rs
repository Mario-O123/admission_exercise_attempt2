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

/// 24 Punkte, nummeriert wie in der Aufgabenstellung
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

/// Parsing für Tests / Beispielspiele:
/// "W P 0" | "B M 3 4" | "W R 5"
impl FromStr for Action {
    type Err = &'static str;

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
        let ak = match parts[1] {
            "P" => {
                let p: Point = parts[2].parse().map_err(|_| "Invalid point")?;
                ActionKind::Place(p)
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
                let p: Point = parts[2].parse().map_err(|_| "Invalid point")?;
                ActionKind::Remove(p)
            }
            _ => return Err("Invalid action type"),
        };
        Ok(Action { player, action: ak })
    }
}

impl Display for Action {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let p = match self.player {
            Player::White => "W",
            Player::Black => "B",
        };
        let s = match self.action {
            ActionKind::Place(x) => format!("P {x}"),
            ActionKind::Move(a, b) => format!("M {a} {b}"),
            ActionKind::Remove(x) => format!("R {x}"),
        };
        write!(f, "{p} {s}")
    }
}

pub trait NmmGame {
    fn new() -> Self;
    fn action(&mut self, action: Action) -> Result<(), &'static str>;
    fn undo(&mut self) -> Result<(), &'static str>;
    fn points(&self) -> &[Option<Piece>; BOARD_POINTS];
    fn winner(&self) -> Option<Player>;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Phase {
    Placing,
    Moving,
    // Flying wird dynamisch pro Spieler gehandhabt (wenn ein Spieler nur 3 Steine hat).
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

pub struct Game {
    board: [Option<Piece>; BOARD_POINTS],
    current_player: Player,
    /// Anzahl tatsächlich gesetzter Steine (nur Placing hochzählen; Removals reduzieren das nicht)
    white_placed: u8,
    black_placed: u8,
    phase: Phase,
    /// Nach einer Mühle: es muss *sofort* ein Remove folgen (durch denselben Spieler)
    pending_removal: bool,
    /// Undo-Stack mit *vollständigen* Snapshots
    history: Vec<GameState>,
}

impl Game {
    fn save_state(&mut self) {
        self.history.push(GameState {
            board: self.board,
            current_player: self.current_player,
            white_placed: self.white_placed,
            black_placed: self.black_placed,
            phase: self.phase,
            pending_removal: self.pending_removal,
        });
    }

    #[inline]
    fn in_bounds(p: Point) -> bool {
        p < BOARD_POINTS
    }

    fn count_pieces(&self, player: Player) -> usize {
        self.board.iter().filter(|s| **s == Some(player)).count()
    }

    fn forms_mill(&self, point: Point, player: Player) -> bool {
        for mill in MILLS.iter() {
            if mill.contains(&point) && mill.iter().all(|&p| self.board[p] == Some(player)) {
                return true;
            }
        }
        false
    }

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

    fn opponent_has_non_mill_piece(&self, opponent: Player) -> bool {
        self.board.iter().enumerate().any(|(i, s)| {
            *s == Some(opponent) && !self.is_part_of_mill(i)
        })
    }

    fn can_player_fly(&self, player: Player) -> bool {
        self.count_pieces(player) == 3
    }

    fn has_legal_move(&self, player: Player) -> bool {
        let player_count = self.count_pieces(player);
        if player_count == 0 {
            return false;
        }
        // In Placing-Phase keine Blockade anhand von Zügen bewerten
        if matches!(self.phase, Phase::Placing) {
            return true;
        }
        // Flying: mindestens ein freies Feld + mindestens ein eigener Stein genügt
        if self.can_player_fly(player) {
            return self.board.iter().any(|s| s.is_none());
        }
        // Normal Moving: irgendein eigener Stein hat einen freien Nachbarn
        for (i, slot) in self.board.iter().enumerate() {
            if *slot == Some(player) && NEIGHBORS[i].iter().any(|&n| self.board[n].is_none()) {
                return true;
            }
        }
        false
    }

    fn maybe_update_phase_after_action(&mut self) {
        // Wechsel von Placing -> Moving, sobald 18 Steine platziert wurden
        if matches!(self.phase, Phase::Placing) && (self.white_placed as u32 + self.black_placed as u32) == 18 {
            // Nur umstellen, wenn gerade *keine* Removal-Pflicht ansteht
            if !self.pending_removal {
                self.phase = Phase::Moving;
            }
        }
        // Flying wird dynamisch in den Regeln für Moves berücksichtigt;
        // Wir behalten phase = Moving bei und erlauben Flying, sobald ein Spieler nur 3 Steine hat.
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
            history: Vec::new(),
        }
    }

    fn action(&mut self, action: Action) -> Result<(), &'static str> {
        // 1) Richtiger Spieler?
        if action.player != self.current_player {
            return Err("Not this player's turn");
        }

        match action.action {
            ActionKind::Place(p) => {
                // Place nur erlaubt, wenn: keine pending_removal und Phase Placing
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

                // Mutationen -> vorher Snapshot
                self.save_state();

                // Setzen
                self.board[p] = Some(self.current_player);
                match self.current_player {
                    Player::White => self.white_placed += 1,
                    Player::Black => self.black_placed += 1,
                }

                // Mill nach Platzierung?
                if self.forms_mill(p, self.current_player) {
                    self.pending_removal = true;
                    // kein Turn-Switch; Removal muss folgen
                } else {
                    self.switch_turn();
                }

                // Phase ggf. umstellen (falls 18 Steine gesetzt und keine Removal-Pflicht)
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

                // Move-Regeln (Flying erlaubt, wenn Spieler nur noch 3 Steine hat)
                let can_fly = self.can_player_fly(self.current_player);
                if !can_fly {
                    // Muss Nachbar sein
                    if !NEIGHBORS[from].contains(&to) {
                        return Err("Not adjacent");
                    }
                }

                // Snapshot
                self.save_state();

                // Ausführen
                self.board[from] = None;
                self.board[to] = Some(self.current_player);

                // Mill nach Zug?
                if self.forms_mill(to, self.current_player) {
                    self.pending_removal = true;
                    // kein Turn-Switch
                } else {
                    self.switch_turn();
                }

                // Phase bleibt Moving; Flying dynamisch
                Ok(())
            }

            ActionKind::Remove(p) => {
                // Remove nur erlaubt, wenn gerade Removal ansteht
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

                // Regel: Wenn es gegnerische Steine außerhalb einer Mühle gibt,
                // darf man keinen Mühlenstein entfernen.
                let opponent = self.current_player.opposite();
                let opp_has_non_mill = self.opponent_has_non_mill_piece(opponent);
                if opp_has_non_mill && self.is_part_of_mill(p) {
                    return Err("Must remove non-mill piece if possible");
                }

                // Snapshot
                self.save_state();

                // Entfernen
                self.board[p] = None;
                self.pending_removal = false;

                // Nach Removal ggf. Phasewechsel prüfen (Placing -> Moving), falls alle 18 gesetzt sind
                self.maybe_update_phase_after_action();

                // Turn an den Gegner
                self.switch_turn();

                Ok(())
            }
        }
    }

    fn undo(&mut self) -> Result<(), &'static str> {
        match self.history.pop() {
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

    fn points(&self) -> &[Option<Piece>; BOARD_POINTS] {
        &self.board
    }

    fn winner(&self) -> Option<Player> {
        // Regel 1: < 3 Steine => verloren
        let w = self.count_pieces(Player::White);
        let b = self.count_pieces(Player::Black);
        if w < 3 {
            return Some(Player::Black);
        }
        if b < 3 {
            return Some(Player::White);
        }

        // In der Placing-Phase nie "keine Züge möglich" werten
        if matches!(self.phase, Phase::Placing) {
            return None;
        }

        // Regel 2: Keine legalen Züge (wenn nicht im Flying)
        // (Ein Spieler mit 3 Steinen kann immer "fliegen", sofern ein freies Feld existiert.)
        if w > 3 && !self.has_legal_move(Player::White) {
            return Some(Player::Black);
        }
        if b > 3 && !self.has_legal_move(Player::Black) {
            return Some(Player::White);
        }

        None
    }
}

// Für lokale Mini-Unit-Tests (optional)
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_board_new_is_empty_local() {
        let g = Game::new();
        assert_eq!(g.points().len(), 24);
        assert!(g.points().iter().all(|p| p.is_none()));
        assert_eq!(g.current_player, Player::White);
        assert!(matches!(g.phase, Phase::Placing));
        assert!(!g.pending_removal);
    }
}
