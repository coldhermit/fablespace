//! One loop: walk the field, talk, fight, and come back.
//!
//! Attack damage is `max(1, atk - def)`. Guard halves that damage, to a
//! minimum of 1. The player's attack includes the starting weapon.
//!
//! Win removes the monster. Flee leaves both of you as you are. A loss
//! restores the player's HP only; the monster stays, at the HP it had,
//! so the fight can be tried again.

use crate::atlas::TILE;

pub const WIDTH: i32 = 1920;
pub const HEIGHT: i32 = 1080;
pub const MAP_W: i32 = 20;
pub const MAP_H: i32 = 11;
/// Field tiles and sprites are drawn at this multiple of the 16px art.
pub const SCALE: i32 = 6;
pub const WALK_FRAMES: u8 = 8;
pub const TRANSITION_FRAMES: u16 = 36;
pub const BEAT: u8 = 28;
pub const LEAVE: u8 = 42;

pub const TALK_LINE: &str = "Keep off my grass.";
pub const COMMANDS: [&str; 3] = ["Attack", "Guard", "Flee"];

pub const WEAPON: Weapon = Weapon {
    name: "Short Sword",
    atk: 3,
};

#[derive(Clone, Copy, Debug)]
pub struct Weapon {
    pub name: &'static str,
    pub atk: i32,
}

#[derive(Clone, Copy, Debug)]
pub struct Fighter {
    pub name: &'static str,
    pub hp: i32,
    pub max_hp: i32,
    pub atk: i32,
    pub def: i32,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Input {
    /// Held direction. -1, 0, or 1. Up and left are negative.
    pub held_x: i32,
    pub held_y: i32,
    /// One-shot direction from a key press. Cleared by the caller each frame.
    pub tap_x: i32,
    pub tap_y: i32,
    pub confirm: bool,
    pub pause: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Back {
    Field,
    Talk,
    Battle,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode {
    Field,
    Talk,
    Transition { frame: u16 },
    Battle,
    Pause(Back),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase {
    Choose,
    AfterPlayer { timer: u8 },
    AfterEnemy { timer: u8 },
    Leave { timer: u8, outcome: Outcome },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Outcome {
    Win,
    Flee,
    Lose,
}

#[derive(Clone, Debug)]
pub struct Battle {
    pub cursor: usize,
    pub log: Vec<String>,
    pub phase: Phase,
    pub guarding: bool,
}

#[derive(Clone, Copy, Debug)]
struct Walk {
    from_x: i32,
    from_y: i32,
    t: u8,
}

pub struct Game {
    pub mode: Mode,
    pub player: Fighter,
    pub monster: Fighter,
    pub monster_alive: bool,
    pub px: i32,
    pub py: i32,
    pub mx: i32,
    pub my: i32,
    pub battle: Battle,
    walk: Option<Walk>,
    queued: Option<(i32, i32)>,
}

impl Game {
    pub fn new() -> Self {
        Self {
            mode: Mode::Field,
            player: Fighter {
                name: "Mira",
                hp: 20,
                max_hp: 20,
                atk: 4,
                def: 2,
            },
            monster: Fighter {
                name: "Slime",
                hp: 12,
                max_hp: 12,
                atk: 5,
                def: 1,
            },
            monster_alive: true,
            px: 2,
            py: 8,
            mx: 8,
            my: 5,
            battle: Battle::fresh("Slime"),
            walk: None,
            queued: None,
        }
    }

    pub fn update(&mut self, input: &Input) {
        if self.handle_pause(input) {
            return;
        }
        match self.mode {
            Mode::Field => {
                if input.confirm && self.near_monster() {
                    self.queued = None;
                    self.mode = Mode::Talk;
                } else {
                    self.move_player(input);
                }
            }
            Mode::Talk => {
                if input.confirm {
                    self.begin_fight();
                }
            }
            Mode::Transition { frame } => {
                let next = frame + 1;
                if next >= TRANSITION_FRAMES {
                    self.mode = Mode::Battle;
                } else {
                    self.mode = Mode::Transition { frame: next };
                }
            }
            Mode::Battle => self.update_battle(input),
            Mode::Pause(_) => {}
        }
    }

    pub fn near_monster(&self) -> bool {
        self.monster_alive
            && self.walk.is_none()
            && (self.px - self.mx).abs() + (self.py - self.my).abs() == 1
    }

    pub fn player_px(&self) -> (i32, i32) {
        match self.walk {
            Some(walk) => {
                let t = i32::from(walk.t);
                let n = i32::from(WALK_FRAMES);
                let x = walk.from_x * TILE + (self.px - walk.from_x) * TILE * t / n;
                let y = walk.from_y * TILE + (self.py - walk.from_y) * TILE * t / n;
                (x, y)
            }
            None => (self.px * TILE, self.py * TILE),
        }
    }

    pub fn bobbing(&self) -> bool {
        self.walk.is_some_and(|walk| walk.t % 8 < 4)
    }

    /// Top-left of the view, in field pixels. A field smaller than the view
    /// sits centered (the origin is negative). A larger map keeps the player
    /// in frame and stops at the edge.
    pub fn camera(&self) -> (i32, i32) {
        let (x, y) = self.player_px();
        let tile = TILE * SCALE;
        let focus_x = x * SCALE + tile / 2;
        let focus_y = y * SCALE + tile / 2;
        (
            scroll_axis(focus_x, MAP_W * tile, WIDTH),
            scroll_axis(focus_y, MAP_H * tile, HEIGHT),
        )
    }

    /// Ground the player can stand on. A wall, or the space past the edge of
    /// a room, is not ground and draws as a flat color.
    pub fn ground(&self, x: i32, y: i32) -> bool {
        x >= 0 && y >= 0 && x < MAP_W && y < MAP_H
    }

    fn handle_pause(&mut self, input: &Input) -> bool {
        let open = input.pause && matches!(self.mode, Mode::Field | Mode::Talk | Mode::Battle);
        let close = matches!(self.mode, Mode::Pause(_)) && (input.pause || input.confirm);
        if !open && !close {
            return false;
        }
        self.mode = match self.mode {
            Mode::Pause(Back::Field) => Mode::Field,
            Mode::Pause(Back::Talk) => Mode::Talk,
            Mode::Pause(Back::Battle) => Mode::Battle,
            Mode::Field => Mode::Pause(Back::Field),
            Mode::Talk => Mode::Pause(Back::Talk),
            Mode::Battle => Mode::Pause(Back::Battle),
            other => other,
        };
        true
    }

    fn begin_fight(&mut self) {
        self.battle = Battle::fresh(self.monster.name);
        self.queued = None;
        self.mode = Mode::Transition { frame: 0 };
    }

    fn move_player(&mut self, input: &Input) {
        self.tick_walk();
        if self.walk.is_some() {
            if input.tap_x != 0 || input.tap_y != 0 {
                self.queued = Some((input.tap_x.signum(), input.tap_y.signum()));
            }
            return;
        }
        let (dx, dy) = if let Some(step) = self.queued.take() {
            step
        } else if input.tap_x != 0 || input.tap_y != 0 {
            (input.tap_x.signum(), input.tap_y.signum())
        } else {
            (input.held_x.signum(), input.held_y.signum())
        };
        if dx != 0 && dy != 0 {
            if self.try_step(dx, dy) {
                return;
            }
            if self.try_step(dx, 0) {
                return;
            }
            let _ = self.try_step(0, dy);
        } else if dx != 0 || dy != 0 {
            let _ = self.try_step(dx, dy);
        }
    }

    fn tick_walk(&mut self) {
        let done = if let Some(walk) = self.walk.as_mut() {
            walk.t = walk.t.saturating_add(1);
            walk.t >= WALK_FRAMES
        } else {
            false
        };
        if done {
            self.walk = None;
        }
    }

    fn try_step(&mut self, dx: i32, dy: i32) -> bool {
        if self.walk.is_some() || (dx == 0 && dy == 0) {
            return false;
        }
        let nx = self.px + dx;
        let ny = self.py + dy;
        if !self.tile_free(nx, ny) {
            return false;
        }
        self.walk = Some(Walk {
            from_x: self.px,
            from_y: self.py,
            t: 0,
        });
        self.px = nx;
        self.py = ny;
        true
    }

    fn tile_free(&self, x: i32, y: i32) -> bool {
        self.ground(x, y) && !(self.monster_alive && x == self.mx && y == self.my)
    }

    fn update_battle(&mut self, input: &Input) {
        match self.battle.phase {
            Phase::Choose => {
                if input.tap_y < 0 {
                    self.battle.cursor = (self.battle.cursor + COMMANDS.len() - 1) % COMMANDS.len();
                } else if input.tap_y > 0 {
                    self.battle.cursor = (self.battle.cursor + 1) % COMMANDS.len();
                }
                if input.confirm {
                    self.player_command();
                }
            }
            Phase::AfterPlayer { timer } => {
                let next = timer.saturating_sub(1);
                if next == 0 {
                    let phase = self.enemy_turn();
                    self.battle.phase = phase;
                } else {
                    self.battle.phase = Phase::AfterPlayer { timer: next };
                }
            }
            Phase::AfterEnemy { timer } => {
                let next = timer.saturating_sub(1);
                self.battle.phase = if next == 0 {
                    Phase::Choose
                } else {
                    Phase::AfterEnemy { timer: next }
                };
            }
            Phase::Leave { timer, outcome } => {
                let next = timer.saturating_sub(1);
                if next == 0 {
                    self.apply_outcome(outcome);
                } else {
                    self.battle.phase = Phase::Leave {
                        timer: next,
                        outcome,
                    };
                }
            }
        }
    }

    fn player_command(&mut self) {
        match self.battle.cursor {
            0 => {
                let dmg = harm(self.player.atk + WEAPON.atk, self.monster.def, false);
                self.monster.hp = (self.monster.hp - dmg).max(0);
                self.push_log(format!("{} hits for {dmg}.", self.player.name));
                if self.monster.hp == 0 {
                    self.push_log(format!("{} melts.", self.monster.name));
                    self.battle.phase = Phase::Leave {
                        timer: LEAVE,
                        outcome: Outcome::Win,
                    };
                } else {
                    self.battle.phase = Phase::AfterPlayer { timer: BEAT };
                }
            }
            1 => {
                self.battle.guarding = true;
                self.push_log(format!("{} guards.", self.player.name));
                self.battle.phase = Phase::AfterPlayer { timer: BEAT };
            }
            _ => {
                self.push_log(format!("{} fled.", self.player.name));
                self.battle.phase = Phase::Leave {
                    timer: LEAVE,
                    outcome: Outcome::Flee,
                };
            }
        }
    }

    fn enemy_turn(&mut self) -> Phase {
        let guard = self.battle.guarding;
        self.battle.guarding = false;
        let dmg = harm(self.monster.atk, self.player.def, guard);
        self.player.hp = (self.player.hp - dmg).max(0);
        self.push_log(format!("{} hits for {dmg}.", self.monster.name));
        if self.player.hp == 0 {
            self.push_log(format!("{} falls.", self.player.name));
            Phase::Leave {
                timer: LEAVE,
                outcome: Outcome::Lose,
            }
        } else {
            Phase::AfterEnemy { timer: BEAT }
        }
    }

    fn apply_outcome(&mut self, outcome: Outcome) {
        match outcome {
            Outcome::Win => self.monster_alive = false,
            Outcome::Flee => {}
            Outcome::Lose => self.player.hp = self.player.max_hp,
        }
        self.queued = None;
        self.walk = None;
        self.mode = Mode::Field;
    }

    fn push_log(&mut self, line: impl Into<String>) {
        self.battle.log.push(line.into());
        let extra = self.battle.log.len().saturating_sub(3);
        if extra > 0 {
            self.battle.log.drain(0..extra);
        }
    }
}

impl Battle {
    fn fresh(monster: &str) -> Self {
        Self {
            cursor: 0,
            log: vec![format!("{monster} draws near.")],
            phase: Phase::Choose,
            guarding: false,
        }
    }
}

fn scroll_axis(focus: i32, span: i32, view: i32) -> i32 {
    if span <= view {
        -(view - span) / 2
    } else {
        (focus - view / 2).clamp(0, span - view)
    }
}

fn harm(atk: i32, def: i32, guard: bool) -> i32 {
    let hit = (atk - def).max(1);
    if guard {
        (hit / 2).max(1)
    } else {
        hit
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn confirm() -> Input {
        Input {
            confirm: true,
            ..Input::default()
        }
    }

    fn idle(game: &mut Game, frames: u16) {
        for _ in 0..frames {
            game.update(&Input::default());
        }
    }

    fn stand_next_to_slime(game: &mut Game) {
        game.px = game.mx;
        game.py = game.my + 1;
        game.walk = None;
    }

    fn enter_battle(game: &mut Game) {
        stand_next_to_slime(game);
        game.update(&confirm());
        assert_eq!(game.mode, Mode::Talk);
        game.update(&confirm());
        assert!(matches!(game.mode, Mode::Transition { .. }));
        idle(game, TRANSITION_FRAMES + 2);
        assert_eq!(game.mode, Mode::Battle);
        assert!(matches!(game.battle.phase, Phase::Choose));
    }

    #[test]
    fn this_field_fills_the_width() {
        assert_eq!(MAP_W * TILE * SCALE, WIDTH);
        assert_eq!(MAP_H * TILE * SCALE, 1056);
        assert!(MAP_H * TILE * SCALE < HEIGHT);
    }

    #[test]
    fn camera_centers_this_field_and_follows_a_larger_map() {
        let game = Game::new();
        let (x, y) = game.camera();
        assert_eq!(x, 0);
        assert_eq!(y, -((HEIGHT - MAP_H * TILE * SCALE) / 2));

        assert_eq!(scroll_axis(0, 4000, WIDTH), 0);
        assert_eq!(scroll_axis(WIDTH / 2, 4000, WIDTH), 0);
        assert_eq!(scroll_axis(2000, 4000, WIDTH), 2000 - WIDTH / 2);
        assert_eq!(scroll_axis(4000, 4000, WIDTH), 4000 - WIDTH);
        assert_eq!(scroll_axis(50, 100, WIDTH), -((WIDTH - 100) / 2));
    }

    #[test]
    fn far_away_confirm_does_nothing() {
        let mut game = Game::new();
        game.update(&confirm());
        assert_eq!(game.mode, Mode::Field);
    }

    #[test]
    fn diagonal_is_not_close_enough_to_talk() {
        let mut game = Game::new();
        game.px = game.mx + 1;
        game.py = game.my + 1;
        game.update(&confirm());
        assert_eq!(game.mode, Mode::Field);
    }

    #[test]
    fn monster_blocks_the_tile() {
        let mut game = Game::new();
        stand_next_to_slime(&mut game);
        game.update(&Input {
            tap_y: -1,
            ..Input::default()
        });
        idle(&mut game, u16::from(WALK_FRAMES) + 2);
        assert_eq!((game.px, game.py), (game.mx, game.my + 1));
    }

    #[test]
    fn a_step_lands_on_the_next_tile() {
        let mut game = Game::new();
        let start = game.px;
        game.update(&Input {
            tap_x: 1,
            ..Input::default()
        });
        assert!(game.walk.is_some());
        idle(&mut game, u16::from(WALK_FRAMES));
        assert_eq!(game.px, start + 1);
        assert!(game.walk.is_none());
    }

    #[test]
    fn pause_opens_and_closes_without_moving() {
        let mut game = Game::new();
        let (px, py) = (game.px, game.py);
        game.update(&Input {
            pause: true,
            ..Input::default()
        });
        assert_eq!(game.mode, Mode::Pause(Back::Field));
        idle(&mut game, 10);
        assert_eq!((game.px, game.py), (px, py));
        game.update(&Input {
            pause: true,
            ..Input::default()
        });
        assert_eq!(game.mode, Mode::Field);
    }

    #[test]
    fn attack_twice_wins_and_removes_the_monster() {
        let mut game = Game::new();
        enter_battle(&mut game);
        for _ in 0..500 {
            if game.mode == Mode::Field {
                break;
            }
            if matches!(game.battle.phase, Phase::Choose) {
                game.update(&confirm());
            } else {
                game.update(&Input::default());
            }
        }
        assert_eq!(game.mode, Mode::Field);
        assert!(!game.monster_alive);
        assert_eq!(game.monster.hp, 0);
        game.update(&confirm());
        assert_eq!(game.mode, Mode::Field);
    }

    #[test]
    fn guard_halves_the_counterattack() {
        let mut game = Game::new();
        enter_battle(&mut game);
        game.update(&Input {
            tap_y: 1,
            ..Input::default()
        });
        assert_eq!(game.battle.cursor, 1);
        let hp = game.player.hp;
        game.update(&confirm());
        idle(&mut game, u16::from(BEAT));
        // Unguarded slime damage is 5 - 2 = 3, so a guard takes 1.
        assert_eq!(game.player.hp, hp - 1);
        assert!(!game.battle.guarding);
    }

    #[test]
    fn flee_leaves_the_monster_and_keeps_hp() {
        let mut game = Game::new();
        enter_battle(&mut game);
        game.update(&confirm());
        idle(&mut game, u16::from(BEAT) * 2 + 4);
        assert!(matches!(game.battle.phase, Phase::Choose));
        assert!(game.monster.hp < game.monster.max_hp);
        assert!(game.player.hp < game.player.max_hp);
        let (player_hp, monster_hp) = (game.player.hp, game.monster.hp);
        game.battle.cursor = 2;
        game.update(&confirm());
        idle(&mut game, u16::from(LEAVE) + 2);
        assert_eq!(game.mode, Mode::Field);
        assert!(game.monster_alive);
        assert_eq!(game.player.hp, player_hp);
        assert_eq!(game.monster.hp, monster_hp);
    }

    #[test]
    fn loss_restores_the_player_and_keeps_the_monster() {
        let mut game = Game::new();
        enter_battle(&mut game);
        game.player.hp = 3;
        game.monster.hp = 8;
        game.update(&confirm());
        idle(&mut game, u16::from(BEAT) + u16::from(LEAVE) + 4);
        assert_eq!(game.mode, Mode::Field);
        assert!(game.monster_alive);
        assert_eq!(game.player.hp, game.player.max_hp);
        assert_eq!(game.monster.hp, 2);
    }
}
