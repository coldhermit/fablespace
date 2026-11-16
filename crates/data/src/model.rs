//! Database, map, and event types.
//!
//! Ids are the index in each list. Index 0 is unused, the same convention as
//! RPG Maker, so an id of 0 in a command means "none".

use serde::{Deserialize, Serialize};

use crate::command::Command;

pub const TILE: u32 = 32;
pub const VIEW_TILES_X: u32 = 17;
pub const VIEW_TILES_Y: u32 = 13;

pub const MHP: usize = 0;
pub const MMP: usize = 1;
pub const ATK: usize = 2;
pub const DEF: usize = 3;
pub const MAT: usize = 4;
pub const MDF: usize = 5;
pub const AGI: usize = 6;
pub const LUK: usize = 7;
pub const PARAM_COUNT: usize = 8;

pub const PARAM_LABELS: [&str; PARAM_COUNT] = [
    "Max HP",
    "Max MP",
    "Attack",
    "Defense",
    "Magic Attack",
    "Magic Defense",
    "Agility",
    "Luck",
];

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum Direction {
    #[default]
    Down,
    Left,
    Right,
    Up,
}

impl Direction {
    pub fn delta(self) -> (i32, i32) {
        match self {
            Direction::Down => (0, 1),
            Direction::Left => (-1, 0),
            Direction::Right => (1, 0),
            Direction::Up => (0, -1),
        }
    }

    pub fn all() -> [Direction; 4] {
        [
            Direction::Down,
            Direction::Left,
            Direction::Right,
            Direction::Up,
        ]
    }
}

impl std::fmt::Display for Direction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Direction::Down => "Down",
            Direction::Left => "Left",
            Direction::Right => "Right",
            Direction::Up => "Up",
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum Passage {
    /// Walkable. On the upper layer this is drawn under the player.
    #[default]
    Pass,
    /// Blocks movement. On the upper layer it sorts with the player.
    Block,
    /// Drawn above the player and never blocks. The tileset star.
    Star,
}

impl Passage {
    pub fn cycle_upper(self) -> Self {
        match self {
            Passage::Pass => Passage::Block,
            Passage::Block => Passage::Star,
            Passage::Star => Passage::Pass,
        }
    }

    pub fn cycle_lower(self) -> Self {
        match self {
            Passage::Block => Passage::Pass,
            _ => Passage::Block,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Passage::Pass => "Pass",
            Passage::Block => "Block",
            Passage::Star => "Star",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum Trigger {
    #[default]
    Action,
    PlayerTouch,
    EventTouch,
    Autorun,
    Parallel,
}

impl std::fmt::Display for Trigger {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Trigger::Action => "Action Button",
            Trigger::PlayerTouch => "Player Touch",
            Trigger::EventTouch => "Event Touch",
            Trigger::Autorun => "Autorun",
            Trigger::Parallel => "Parallel",
        })
    }
}

impl Trigger {
    pub fn all() -> [Trigger; 5] {
        [
            Trigger::Action,
            Trigger::PlayerTouch,
            Trigger::EventTouch,
            Trigger::Autorun,
            Trigger::Parallel,
        ]
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum Priority {
    Below,
    #[default]
    Same,
    Above,
}

impl std::fmt::Display for Priority {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Priority::Below => "Below characters",
            Priority::Same => "Same as characters",
            Priority::Above => "Above characters",
        })
    }
}

impl Priority {
    pub fn all() -> [Priority; 3] {
        [Priority::Below, Priority::Same, Priority::Above]
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum MoveType {
    #[default]
    Fixed,
    Random,
    Approach,
    Custom,
}

impl std::fmt::Display for MoveType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            MoveType::Fixed => "Fixed",
            MoveType::Random => "Random",
            MoveType::Approach => "Approach",
            MoveType::Custom => "Custom",
        })
    }
}

impl MoveType {
    pub fn all() -> [MoveType; 4] {
        [
            MoveType::Fixed,
            MoveType::Random,
            MoveType::Approach,
            MoveType::Custom,
        ]
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SelfSwitch {
    A,
    B,
    C,
    D,
}

impl SelfSwitch {
    pub fn all() -> [SelfSwitch; 4] {
        [SelfSwitch::A, SelfSwitch::B, SelfSwitch::C, SelfSwitch::D]
    }

    pub fn index(self) -> u8 {
        match self {
            SelfSwitch::A => 0,
            SelfSwitch::B => 1,
            SelfSwitch::C => 2,
            SelfSwitch::D => 3,
        }
    }

    pub fn from_index(index: u8) -> Option<Self> {
        match index {
            0 => Some(SelfSwitch::A),
            1 => Some(SelfSwitch::B),
            2 => Some(SelfSwitch::C),
            3 => Some(SelfSwitch::D),
            _ => None,
        }
    }
}

impl std::fmt::Display for SelfSwitch {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            SelfSwitch::A => "A",
            SelfSwitch::B => "B",
            SelfSwitch::C => "C",
            SelfSwitch::D => "D",
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum Occasion {
    #[default]
    Always,
    Battle,
    Menu,
}

impl std::fmt::Display for Occasion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Occasion::Always => "Always",
            Occasion::Battle => "Battle",
            Occasion::Menu => "Menu",
        })
    }
}

impl Occasion {
    pub fn all() -> [Occasion; 3] {
        [Occasion::Always, Occasion::Battle, Occasion::Menu]
    }

    pub fn allows_battle(self) -> bool {
        matches!(self, Occasion::Always | Occasion::Battle)
    }

    pub fn allows_menu(self) -> bool {
        matches!(self, Occasion::Always | Occasion::Menu)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum Scope {
    #[default]
    None,
    OneEnemy,
    AllEnemies,
    OneAlly,
    AllAllies,
    User,
}

impl std::fmt::Display for Scope {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Scope::None => "None",
            Scope::OneEnemy => "One Enemy",
            Scope::AllEnemies => "All Enemies",
            Scope::OneAlly => "One Ally",
            Scope::AllAllies => "All Allies",
            Scope::User => "User",
        })
    }
}

impl Scope {
    pub fn all() -> [Scope; 6] {
        [
            Scope::None,
            Scope::OneEnemy,
            Scope::AllEnemies,
            Scope::OneAlly,
            Scope::AllAllies,
            Scope::User,
        ]
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum DamageKind {
    #[default]
    None,
    HpDamage,
    MpDamage,
    HpRecover,
    MpRecover,
}

impl std::fmt::Display for DamageKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            DamageKind::None => "None",
            DamageKind::HpDamage => "HP Damage",
            DamageKind::MpDamage => "MP Damage",
            DamageKind::HpRecover => "HP Recover",
            DamageKind::MpRecover => "MP Recover",
        })
    }
}

impl DamageKind {
    pub fn all() -> [DamageKind; 5] {
        [
            DamageKind::None,
            DamageKind::HpDamage,
            DamageKind::MpDamage,
            DamageKind::HpRecover,
            DamageKind::MpRecover,
        ]
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum Element {
    #[default]
    Physical,
    Magical,
}

impl std::fmt::Display for Element {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Element::Physical => "Physical",
            Element::Magical => "Magical",
        })
    }
}

impl Element {
    pub fn all() -> [Element; 2] {
        [Element::Physical, Element::Magical]
    }
}

/// Fixed shape of the RPG Maker default formula.
/// Physical damage is `power + a.atk * 4 - b.def * 2`.
/// Magical damage uses Magic Attack and Magic Defense.
/// Magical recovery is `power + a.mat * 2`. Physical recovery is `power`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Damage {
    pub kind: DamageKind,
    pub element: Element,
    pub power: i32,
    pub variance: i32,
}

impl Default for Damage {
    fn default() -> Self {
        Self {
            kind: DamageKind::None,
            element: Element::Physical,
            power: 0,
            variance: 20,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Effect {
    RecoverHp { percent: i32, flat: i32 },
    RecoverMp { percent: i32, flat: i32 },
    AddState { state_id: u32, chance: i32 },
    RemoveState { state_id: u32 },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum Restriction {
    #[default]
    None,
    CannotAct,
}

impl std::fmt::Display for Restriction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Restriction::None => "None",
            Restriction::CannotAct => "Cannot act",
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum Removal {
    #[default]
    None,
    Turns {
        min: u32,
        max: u32,
    },
    OnDamage,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ArmorSlot {
    Shield,
    Head,
    #[default]
    Body,
    Accessory,
}

impl std::fmt::Display for ArmorSlot {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            ArmorSlot::Shield => "Shield",
            ArmorSlot::Head => "Head",
            ArmorSlot::Body => "Body",
            ArmorSlot::Accessory => "Accessory",
        })
    }
}

impl ArmorSlot {
    pub fn all() -> [ArmorSlot; 4] {
        [
            ArmorSlot::Shield,
            ArmorSlot::Head,
            ArmorSlot::Body,
            ArmorSlot::Accessory,
        ]
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum CommonTrigger {
    #[default]
    None,
    Autorun,
    Parallel,
}

impl std::fmt::Display for CommonTrigger {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            CommonTrigger::None => "None",
            CommonTrigger::Autorun => "Autorun",
            CommonTrigger::Parallel => "Parallel",
        })
    }
}

impl CommonTrigger {
    pub fn all() -> [CommonTrigger; 3] {
        [
            CommonTrigger::None,
            CommonTrigger::Autorun,
            CommonTrigger::Parallel,
        ]
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ParamGrowth {
    pub base: i32,
    pub plus: i32,
}

impl Default for ParamGrowth {
    fn default() -> Self {
        Self { base: 10, plus: 1 }
    }
}

impl ParamGrowth {
    pub fn at(self, level: u32) -> i32 {
        let level = level.max(1) as i32;
        self.base + self.plus * (level - 1)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExpCurve {
    pub basis: i32,
    pub extra: i32,
    pub accel: i32,
}

impl Default for ExpCurve {
    fn default() -> Self {
        Self {
            basis: 20,
            extra: 10,
            accel: 5,
        }
    }
}

/// Experience required to leave `level`.
/// `basis + extra * (level - 1) + accel * (level - 1)^2 / 10`.
pub fn exp_to_next(curve: &ExpCurve, level: u32) -> i32 {
    let l = level.max(1) as i32 - 1;
    (curve.basis + curve.extra * l + curve.accel * l * l / 10).max(1)
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Learning {
    pub level: u32,
    pub skill_id: u32,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct Terms {
    pub hp: String,
    pub mp: String,
    pub level: String,
    pub attack: String,
    pub guard: String,
    pub item: String,
    pub skill: String,
    pub equip: String,
    pub status: String,
    pub save: String,
    pub game_end: String,
    pub new_game: String,
    pub continue_game: String,
    pub shutdown: String,
    pub fight: String,
    pub escape: String,
    pub to_title: String,
    pub gold: String,
}

impl Terms {
    pub fn standard() -> Self {
        Self {
            hp: "HP".into(),
            mp: "MP".into(),
            level: "Level".into(),
            attack: "Attack".into(),
            guard: "Guard".into(),
            item: "Item".into(),
            skill: "Skill".into(),
            equip: "Equip".into(),
            status: "Status".into(),
            save: "Save".into(),
            game_end: "End Game".into(),
            new_game: "New Game".into(),
            continue_game: "Continue".into(),
            shutdown: "Exit".into(),
            fight: "Fight".into(),
            escape: "Escape".into(),
            to_title: "To Title".into(),
            gold: "Gold".into(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Stack {
    pub id: u32,
    pub count: u32,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct System {
    pub game_title: String,
    pub currency: String,
    pub start_map_id: u32,
    pub start_x: u32,
    pub start_y: u32,
    pub start_dir: Direction,
    pub party: Vec<u32>,
    pub start_gold: i32,
    pub start_items: Vec<Stack>,
    pub start_weapons: Vec<Stack>,
    pub start_armors: Vec<Stack>,
    pub title_image: String,
    pub terms: Terms,
}

impl Default for System {
    fn default() -> Self {
        Self {
            game_title: "Untitled".into(),
            currency: "G".into(),
            start_map_id: 1,
            start_x: 1,
            start_y: 1,
            start_dir: Direction::Down,
            party: vec![1],
            start_gold: 0,
            start_items: Vec::new(),
            start_weapons: Vec::new(),
            start_armors: Vec::new(),
            title_image: "title".into(),
            terms: Terms::standard(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Actor {
    pub id: u32,
    pub name: String,
    pub nickname: String,
    pub class_id: u32,
    pub initial_level: u32,
    pub max_level: u32,
    pub profile: String,
    pub face: String,
    pub character: String,
    pub character_index: u32,
    pub weapon_id: u32,
    pub shield_id: u32,
    pub head_id: u32,
    pub body_id: u32,
    pub accessory_id: u32,
    pub note: String,
}

impl Default for Actor {
    fn default() -> Self {
        Self {
            id: 0,
            name: String::new(),
            nickname: String::new(),
            class_id: 1,
            initial_level: 1,
            max_level: 99,
            profile: String::new(),
            face: String::new(),
            character: String::new(),
            character_index: 0,
            weapon_id: 0,
            shield_id: 0,
            head_id: 0,
            body_id: 0,
            accessory_id: 0,
            note: String::new(),
        }
    }
}

impl Actor {
    pub fn blank(id: u32) -> Self {
        Self {
            id,
            name: format!("Actor {id}"),
            character: "hero".into(),
            ..Self::default()
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Class {
    pub id: u32,
    pub name: String,
    pub exp: ExpCurve,
    pub params: Vec<ParamGrowth>,
    pub learnings: Vec<Learning>,
    pub note: String,
}

impl Default for Class {
    fn default() -> Self {
        Self {
            id: 0,
            name: String::new(),
            exp: ExpCurve::default(),
            params: vec![ParamGrowth::default(); PARAM_COUNT],
            learnings: Vec::new(),
            note: String::new(),
        }
    }
}

impl Class {
    pub fn blank(id: u32) -> Self {
        Self {
            id,
            name: format!("Class {id}"),
            params: vec![
                ParamGrowth { base: 40, plus: 6 },
                ParamGrowth { base: 10, plus: 2 },
                ParamGrowth { base: 6, plus: 1 },
                ParamGrowth { base: 5, plus: 1 },
                ParamGrowth { base: 5, plus: 1 },
                ParamGrowth { base: 5, plus: 1 },
                ParamGrowth { base: 6, plus: 1 },
                ParamGrowth { base: 4, plus: 0 },
            ],
            ..Self::default()
        }
    }

    pub fn growth(&self, param: usize) -> ParamGrowth {
        self.params
            .get(param)
            .cloned()
            .unwrap_or(ParamGrowth { base: 1, plus: 0 })
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Skill {
    pub id: u32,
    pub name: String,
    pub description: String,
    pub mp_cost: i32,
    pub occasion: Occasion,
    pub scope: Scope,
    pub hit: i32,
    pub damage: Damage,
    pub effects: Vec<Effect>,
    pub note: String,
}

impl Default for Skill {
    fn default() -> Self {
        Self {
            id: 0,
            name: String::new(),
            description: String::new(),
            mp_cost: 0,
            occasion: Occasion::Always,
            scope: Scope::None,
            hit: 100,
            damage: Damage::default(),
            effects: Vec::new(),
            note: String::new(),
        }
    }
}

impl Skill {
    pub fn blank(id: u32) -> Self {
        Self {
            id,
            name: format!("Skill {id}"),
            hit: 100,
            ..Self::default()
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Item {
    pub id: u32,
    pub name: String,
    pub description: String,
    pub price: i32,
    pub consumable: bool,
    pub occasion: Occasion,
    pub scope: Scope,
    pub hit: i32,
    pub damage: Damage,
    pub effects: Vec<Effect>,
    pub note: String,
}

impl Default for Item {
    fn default() -> Self {
        Self {
            id: 0,
            name: String::new(),
            description: String::new(),
            price: 0,
            consumable: true,
            occasion: Occasion::Always,
            scope: Scope::User,
            hit: 100,
            damage: Damage::default(),
            effects: Vec::new(),
            note: String::new(),
        }
    }
}

impl Item {
    pub fn blank(id: u32) -> Self {
        Self {
            id,
            name: format!("Item {id}"),
            ..Self::default()
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Weapon {
    pub id: u32,
    pub name: String,
    pub description: String,
    pub price: i32,
    pub params: Vec<i32>,
    pub note: String,
}

impl Default for Weapon {
    fn default() -> Self {
        Self {
            id: 0,
            name: String::new(),
            description: String::new(),
            price: 0,
            params: vec![0; PARAM_COUNT],
            note: String::new(),
        }
    }
}

impl Weapon {
    pub fn blank(id: u32) -> Self {
        Self {
            id,
            name: format!("Weapon {id}"),
            ..Self::default()
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Armor {
    pub id: u32,
    pub name: String,
    pub description: String,
    pub price: i32,
    pub slot: ArmorSlot,
    pub params: Vec<i32>,
    pub note: String,
}

impl Default for Armor {
    fn default() -> Self {
        Self {
            id: 0,
            name: String::new(),
            description: String::new(),
            price: 0,
            slot: ArmorSlot::Body,
            params: vec![0; PARAM_COUNT],
            note: String::new(),
        }
    }
}

impl Armor {
    pub fn blank(id: u32) -> Self {
        Self {
            id,
            name: format!("Armor {id}"),
            ..Self::default()
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ActionCondition {
    Always,
    HpBelow { percent: i32 },
    Turn { start: u32, every: u32 },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EnemyAction {
    /// Skill id, or 0 for a basic attack.
    pub skill_id: u32,
    pub rating: u32,
    pub condition: ActionCondition,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Drop {
    pub item_id: u32,
    /// Chance is `1 / denominator`. A denominator of 1 is a certain drop.
    pub denominator: u32,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Enemy {
    pub id: u32,
    pub name: String,
    pub battler: String,
    pub params: Vec<i32>,
    pub exp: i32,
    pub gold: i32,
    pub drops: Vec<Drop>,
    pub actions: Vec<EnemyAction>,
    pub note: String,
}

impl Default for Enemy {
    fn default() -> Self {
        Self {
            id: 0,
            name: String::new(),
            battler: String::new(),
            params: vec![10, 0, 5, 5, 5, 5, 5, 4],
            exp: 0,
            gold: 0,
            drops: Vec::new(),
            actions: Vec::new(),
            note: String::new(),
        }
    }
}

impl Enemy {
    pub fn blank(id: u32) -> Self {
        Self {
            id,
            name: format!("Enemy {id}"),
            battler: "slime".into(),
            params: vec![30, 0, 8, 4, 4, 4, 6, 4],
            exp: 10,
            gold: 5,
            actions: vec![EnemyAction {
                skill_id: 0,
                rating: 5,
                condition: ActionCondition::Always,
            }],
            ..Self::default()
        }
    }

    pub fn param(&self, index: usize) -> i32 {
        self.params.get(index).copied().unwrap_or(0).max(0)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TroopMember {
    pub enemy_id: u32,
    pub x: i32,
    pub y: i32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum Span {
    #[default]
    Battle,
    Turn,
}

impl std::fmt::Display for Span {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Span::Battle => "Battle",
            Span::Turn => "Turn",
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct TroopCondition {
    /// When set, the page runs on this turn. `every` of 0 means that turn only.
    pub turn_start: Option<u32>,
    pub turn_every: u32,
    /// Troop member index and HP percent.
    pub enemy_hp: Option<(usize, i32)>,
    pub switch_id: u32,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TroopPage {
    pub condition: TroopCondition,
    pub span: Span,
    pub list: Vec<Command>,
}

impl Default for TroopPage {
    fn default() -> Self {
        Self {
            condition: TroopCondition::default(),
            span: Span::Battle,
            list: Vec::new(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Troop {
    pub id: u32,
    pub name: String,
    pub members: Vec<TroopMember>,
    pub pages: Vec<TroopPage>,
}

impl Default for Troop {
    fn default() -> Self {
        Self {
            id: 0,
            name: String::new(),
            members: Vec::new(),
            pages: Vec::new(),
        }
    }
}

impl Troop {
    pub fn blank(id: u32) -> Self {
        Self {
            id,
            name: format!("Troop {id}"),
            members: vec![TroopMember {
                enemy_id: 1,
                x: 200,
                y: 110,
            }],
            pages: Vec::new(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct State {
    pub id: u32,
    pub name: String,
    pub restriction: Restriction,
    pub removal: Removal,
    /// Percent of max HP removed at the end of a turn.
    pub slip_hp_percent: i32,
    pub note: String,
}

impl Default for State {
    fn default() -> Self {
        Self {
            id: 0,
            name: String::new(),
            restriction: Restriction::None,
            removal: Removal::None,
            slip_hp_percent: 0,
            note: String::new(),
        }
    }
}

impl State {
    pub fn blank(id: u32) -> Self {
        Self {
            id,
            name: format!("State {id}"),
            ..Self::default()
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TileDef {
    pub passage: Passage,
}

impl Default for TileDef {
    fn default() -> Self {
        Self {
            passage: Passage::Pass,
        }
    }
}

/// A 4-neighbor autotile. The sixteen variants sit in a 4 by 4 block whose
/// top-left cell is the id painted onto the map.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Autotile {
    pub tile_id: u32,
    pub origin_col: u32,
    pub origin_row: u32,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Tileset {
    pub id: u32,
    pub name: String,
    pub image: String,
    pub columns: u32,
    pub tiles: Vec<TileDef>,
    pub autotiles: Vec<Autotile>,
    pub note: String,
}

impl Default for Tileset {
    fn default() -> Self {
        Self {
            id: 0,
            name: String::new(),
            image: String::new(),
            columns: 8,
            tiles: vec![TileDef::default()],
            autotiles: Vec::new(),
            note: String::new(),
        }
    }
}

impl Tileset {
    pub fn blank(id: u32) -> Self {
        let mut tiles = vec![TileDef {
            passage: Passage::Pass,
        }];
        // A small default set matching the generated meadow sheet.
        let passages = [
            Passage::Pass,  // 1 grass
            Passage::Pass,  // 2 stone
            Passage::Block, // 3 water
            Passage::Pass,  // 4 flower
            Passage::Block, // 5 tree
        ];
        for passage in passages {
            tiles.push(TileDef { passage });
        }
        while tiles.len() < 40 {
            tiles.push(TileDef {
                passage: Passage::Pass,
            });
        }
        Self {
            id,
            name: format!("Tileset {id}"),
            image: "meadow".into(),
            columns: 8,
            tiles,
            autotiles: vec![Autotile {
                tile_id: 8,
                origin_col: 0,
                origin_row: 1,
            }],
            note: String::new(),
        }
    }

    pub fn passage(&self, id: u32) -> Passage {
        self.tiles
            .get(id as usize)
            .map(|t| t.passage)
            .unwrap_or(Passage::Block)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CommonEvent {
    pub id: u32,
    pub name: String,
    pub trigger: CommonTrigger,
    pub switch_id: u32,
    pub list: Vec<Command>,
}

impl Default for CommonEvent {
    fn default() -> Self {
        Self {
            id: 0,
            name: String::new(),
            trigger: CommonTrigger::None,
            switch_id: 0,
            list: Vec::new(),
        }
    }
}

impl CommonEvent {
    pub fn blank(id: u32) -> Self {
        Self {
            id,
            name: format!("Common Event {id}"),
            ..Self::default()
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct PageConditions {
    pub switch_1: u32,
    pub switch_2: u32,
    pub variable_id: u32,
    pub variable_min: i32,
    pub self_switch: Option<SelfSwitch>,
    pub item_id: u32,
    pub actor_id: u32,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EventImage {
    pub character: String,
    pub index: u32,
    pub direction: Direction,
    pub pattern: u8,
}

impl Default for EventImage {
    fn default() -> Self {
        Self {
            character: String::new(),
            index: 0,
            direction: Direction::Down,
            pattern: 1,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Page {
    pub conditions: PageConditions,
    pub image: EventImage,
    pub move_type: MoveType,
    pub move_speed: u8,
    pub move_frequency: u8,
    pub route: crate::command::MoveRoute,
    pub walk_anime: bool,
    pub step_anime: bool,
    pub direction_fix: bool,
    pub through: bool,
    pub priority: Priority,
    pub trigger: Trigger,
    pub list: Vec<Command>,
}

impl Default for Page {
    fn default() -> Self {
        Self {
            conditions: PageConditions::default(),
            image: EventImage::default(),
            move_type: MoveType::Fixed,
            move_speed: 4,
            move_frequency: 3,
            route: crate::command::MoveRoute::default(),
            walk_anime: true,
            step_anime: false,
            direction_fix: false,
            through: false,
            priority: Priority::Same,
            trigger: Trigger::Action,
            list: Vec::new(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Event {
    pub id: u32,
    pub name: String,
    pub x: u32,
    pub y: u32,
    pub pages: Vec<Page>,
}

impl Event {
    pub fn blank(id: u32, x: u32, y: u32) -> Self {
        Self {
            id,
            name: format!("EV{id:03}"),
            x,
            y,
            pages: vec![Page::default()],
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Map {
    pub id: u32,
    pub name: String,
    pub width: u32,
    pub height: u32,
    pub tileset_id: u32,
    pub lower: Vec<u32>,
    pub upper: Vec<u32>,
    pub events: Vec<Event>,
}

impl Map {
    pub fn blank(
        id: u32,
        name: impl Into<String>,
        width: u32,
        height: u32,
        tileset_id: u32,
        fill: u32,
    ) -> Self {
        let width = width.clamp(1, 256);
        let height = height.clamp(1, 256);
        let n = (width * height) as usize;
        Self {
            id,
            name: name.into(),
            width,
            height,
            tileset_id,
            lower: vec![fill; n],
            upper: vec![0; n],
            events: Vec::new(),
        }
    }

    pub fn index(&self, x: i32, y: i32) -> Option<usize> {
        if x < 0 || y < 0 || x >= self.width as i32 || y >= self.height as i32 {
            return None;
        }
        Some((y as u32 * self.width + x as u32) as usize)
    }

    pub fn lower_at(&self, x: i32, y: i32) -> u32 {
        self.index(x, y)
            .and_then(|i| self.lower.get(i).copied())
            .unwrap_or(0)
    }

    pub fn upper_at(&self, x: i32, y: i32) -> u32 {
        self.index(x, y)
            .and_then(|i| self.upper.get(i).copied())
            .unwrap_or(0)
    }

    pub fn set_lower(&mut self, x: u32, y: u32, tile: u32) {
        if let Some(i) = self.index(x as i32, y as i32) {
            if i < self.lower.len() {
                self.lower[i] = tile;
            }
        }
    }

    pub fn set_upper(&mut self, x: u32, y: u32, tile: u32) {
        if let Some(i) = self.index(x as i32, y as i32) {
            if i < self.upper.len() {
                self.upper[i] = tile;
            }
        }
    }

    pub fn resize(&mut self, width: u32, height: u32, fill: u32) {
        let width = width.clamp(1, 256);
        let height = height.clamp(1, 256);
        let mut lower = vec![fill; (width * height) as usize];
        let mut upper = vec![0; (width * height) as usize];
        for y in 0..height.min(self.height) {
            for x in 0..width.min(self.width) {
                let old = (y * self.width + x) as usize;
                let new = (y * width + x) as usize;
                lower[new] = self.lower.get(old).copied().unwrap_or(fill);
                upper[new] = self.upper.get(old).copied().unwrap_or(0);
            }
        }
        self.events.retain(|e| e.x < width && e.y < height);
        self.width = width;
        self.height = height;
        self.lower = lower;
        self.upper = upper;
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Game {
    pub system: System,
    pub actors: Vec<Actor>,
    pub classes: Vec<Class>,
    pub skills: Vec<Skill>,
    pub items: Vec<Item>,
    pub weapons: Vec<Weapon>,
    pub armors: Vec<Armor>,
    pub enemies: Vec<Enemy>,
    pub troops: Vec<Troop>,
    pub states: Vec<State>,
    pub tilesets: Vec<Tileset>,
    pub common_events: Vec<CommonEvent>,
    pub switches: Vec<String>,
    pub variables: Vec<String>,
}

impl Default for Game {
    fn default() -> Self {
        Self {
            system: System::default(),
            actors: vec![Actor::default()],
            classes: vec![Class::default()],
            skills: vec![Skill::default()],
            items: vec![Item::default()],
            weapons: vec![Weapon::default()],
            armors: vec![Armor::default()],
            enemies: vec![Enemy::default()],
            troops: vec![Troop::default()],
            states: vec![State::default()],
            tilesets: vec![Tileset::default()],
            common_events: vec![CommonEvent::default()],
            switches: vec![String::new(), "Switch 1".into()],
            variables: vec![String::new(), "Variable 1".into()],
        }
    }
}

impl Game {
    pub fn actor(&self, id: u32) -> Option<&Actor> {
        self.get(id, &self.actors)
    }
    pub fn class(&self, id: u32) -> Option<&Class> {
        self.get(id, &self.classes)
    }
    pub fn skill(&self, id: u32) -> Option<&Skill> {
        self.get(id, &self.skills)
    }
    pub fn item(&self, id: u32) -> Option<&Item> {
        self.get(id, &self.items)
    }
    pub fn weapon(&self, id: u32) -> Option<&Weapon> {
        self.get(id, &self.weapons)
    }
    pub fn armor(&self, id: u32) -> Option<&Armor> {
        self.get(id, &self.armors)
    }
    pub fn enemy(&self, id: u32) -> Option<&Enemy> {
        self.get(id, &self.enemies)
    }
    pub fn troop(&self, id: u32) -> Option<&Troop> {
        self.get(id, &self.troops)
    }
    pub fn state(&self, id: u32) -> Option<&State> {
        self.get(id, &self.states)
    }
    pub fn tileset(&self, id: u32) -> Option<&Tileset> {
        self.get(id, &self.tilesets)
    }
    pub fn common_event(&self, id: u32) -> Option<&CommonEvent> {
        self.get(id, &self.common_events)
    }

    fn get<'a, T>(&'a self, id: u32, list: &'a [T]) -> Option<&'a T> {
        if id == 0 {
            None
        } else {
            list.get(id as usize)
        }
    }

    pub fn normalize(&mut self) {
        fn fix<T>(list: &mut Vec<T>, set: impl Fn(&mut T, u32)) {
            if list.is_empty() {
                return;
            }
            for (i, item) in list.iter_mut().enumerate() {
                set(item, i as u32);
            }
        }
        fix(&mut self.actors, |a, i| a.id = i);
        fix(&mut self.classes, |a, i| a.id = i);
        fix(&mut self.skills, |a, i| a.id = i);
        fix(&mut self.items, |a, i| a.id = i);
        fix(&mut self.weapons, |a, i| a.id = i);
        fix(&mut self.armors, |a, i| a.id = i);
        fix(&mut self.enemies, |a, i| a.id = i);
        fix(&mut self.troops, |a, i| a.id = i);
        fix(&mut self.states, |a, i| a.id = i);
        fix(&mut self.tilesets, |a, i| a.id = i);
        fix(&mut self.common_events, |a, i| a.id = i);
        if self.switches.is_empty() {
            self.switches.push(String::new());
        }
        if self.variables.is_empty() {
            self.variables.push(String::new());
        }
        self.switches[0].clear();
        self.variables[0].clear();
    }

    pub fn switch_name(&self, id: u32) -> String {
        self.switches
            .get(id as usize)
            .filter(|s| !s.is_empty())
            .cloned()
            .unwrap_or_else(|| format!("Switch {id}"))
    }

    pub fn variable_name(&self, id: u32) -> String {
        self.variables
            .get(id as usize)
            .filter(|s| !s.is_empty())
            .cloned()
            .unwrap_or_else(|| format!("Variable {id}"))
    }
}
