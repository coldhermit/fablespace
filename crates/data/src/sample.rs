//! Fable Hollow, the project a fresh clone can play.

use crate::command::{ActorTarget, AmountOp, CancelMode, Command, Operand, SwitchValue, VarOp};
use crate::model::*;

pub const GRASS: u32 = 1;
pub const STONE: u32 = 2;
pub const WATER: u32 = 3;
pub const FLOWER: u32 = 4;
pub const TREE: u32 = 5;
pub const PATH: u32 = 8;

pub fn fable_hollow() -> (Game, Vec<Map>) {
    let mut game = Game {
        system: System {
            game_title: "Fable Hollow".into(),
            currency: "G".into(),
            start_map_id: 1,
            start_x: 3,
            start_y: 12,
            start_dir: Direction::Up,
            party: vec![1],
            start_gold: 20,
            start_items: vec![Stack { id: 1, count: 1 }],
            start_weapons: vec![Stack { id: 2, count: 1 }],
            start_armors: vec![Stack { id: 2, count: 1 }],
            title_image: "title".into(),
            terms: Terms::standard(),
        },
        actors: vec![
            Actor::default(),
            Actor {
                id: 1,
                name: "Lark".into(),
                nickname: "Wayfarer".into(),
                class_id: 1,
                initial_level: 1,
                max_level: 20,
                profile: "A traveler who keeps to the meadow paths.".into(),
                face: "lark".into(),
                character: "hero".into(),
                character_index: 0,
                weapon_id: 1,
                body_id: 1,
                note: String::new(),
                ..Actor::default()
            },
        ],
        classes: vec![
            Class::default(),
            Class {
                id: 1,
                name: "Wayfarer".into(),
                exp: ExpCurve {
                    basis: 20,
                    extra: 8,
                    accel: 4,
                },
                params: vec![
                    ParamGrowth { base: 48, plus: 8 },
                    ParamGrowth { base: 16, plus: 2 },
                    ParamGrowth { base: 5, plus: 1 },
                    ParamGrowth { base: 5, plus: 1 },
                    ParamGrowth { base: 7, plus: 1 },
                    ParamGrowth { base: 5, plus: 1 },
                    ParamGrowth { base: 7, plus: 1 },
                    ParamGrowth { base: 4, plus: 0 },
                ],
                learnings: vec![
                    Learning {
                        level: 1,
                        skill_id: 1,
                    },
                    Learning {
                        level: 1,
                        skill_id: 2,
                    },
                ],
                note: String::new(),
            },
        ],
        skills: vec![
            Skill::default(),
            Skill {
                id: 1,
                name: "Mend".into(),
                description: "Close a cut with a warm light.".into(),
                mp_cost: 5,
                occasion: Occasion::Always,
                scope: Scope::OneAlly,
                hit: 100,
                damage: Damage {
                    kind: DamageKind::HpRecover,
                    element: Element::Magical,
                    power: 18,
                    variance: 0,
                },
                effects: Vec::new(),
                note: String::new(),
            },
            Skill {
                id: 2,
                name: "Spark".into(),
                description: "A crack of static.".into(),
                mp_cost: 6,
                occasion: Occasion::Battle,
                scope: Scope::OneEnemy,
                hit: 95,
                damage: Damage {
                    kind: DamageKind::HpDamage,
                    element: Element::Magical,
                    power: 6,
                    variance: 10,
                },
                effects: Vec::new(),
                note: String::new(),
            },
            Skill {
                id: 3,
                name: "Ooze".into(),
                description: "A slick that keeps burning.".into(),
                mp_cost: 0,
                occasion: Occasion::Battle,
                scope: Scope::OneEnemy,
                hit: 90,
                damage: Damage {
                    kind: DamageKind::None,
                    element: Element::Magical,
                    power: 0,
                    variance: 0,
                },
                effects: vec![Effect::AddState {
                    state_id: 2,
                    chance: 100,
                }],
                note: String::new(),
            },
        ],
        items: vec![
            Item::default(),
            Item {
                id: 1,
                name: "Potion".into(),
                description: "A corked vial of meadow tonic.".into(),
                price: 25,
                consumable: true,
                occasion: Occasion::Always,
                scope: Scope::OneAlly,
                hit: 100,
                damage: Damage::default(),
                effects: vec![Effect::RecoverHp {
                    percent: 0,
                    flat: 30,
                }],
                note: String::new(),
            },
            Item {
                id: 2,
                name: "Berry".into(),
                description: "Tart, and better than nothing.".into(),
                price: 8,
                consumable: true,
                occasion: Occasion::Always,
                scope: Scope::OneAlly,
                hit: 100,
                damage: Damage::default(),
                effects: vec![Effect::RecoverHp {
                    percent: 0,
                    flat: 15,
                }],
                note: String::new(),
            },
        ],
        weapons: vec![
            Weapon::default(),
            Weapon {
                id: 1,
                name: "Walking Stick".into(),
                description: "A straight branch, sanded smooth.".into(),
                price: 40,
                params: params_with(ATK, 2),
                note: String::new(),
            },
            Weapon {
                id: 2,
                name: "Thorn".into(),
                description: "A short knife with a wrapped handle.".into(),
                price: 30,
                params: params_with(ATK, 1),
                note: String::new(),
            },
        ],
        armors: vec![
            Armor::default(),
            Armor {
                id: 1,
                name: "Cloth Shirt".into(),
                description: "Mended at the elbows.".into(),
                price: 30,
                slot: ArmorSlot::Body,
                params: params_with(DEF, 3),
                note: String::new(),
            },
            Armor {
                id: 2,
                name: "Leaf Cloak".into(),
                description: "Woven to shed a light rain.".into(),
                price: 20,
                slot: ArmorSlot::Body,
                params: params_with(DEF, 1),
                note: String::new(),
            },
        ],
        enemies: vec![
            Enemy::default(),
            Enemy {
                id: 1,
                name: "Meadow Slime".into(),
                battler: "slime".into(),
                params: vec![40, 0, 8, 6, 2, 4, 5, 3],
                exp: 30,
                gold: 12,
                drops: vec![Drop {
                    item_id: 2,
                    denominator: 2,
                }],
                actions: vec![
                    EnemyAction {
                        skill_id: 0,
                        rating: 5,
                        condition: ActionCondition::Always,
                    },
                    EnemyAction {
                        skill_id: 3,
                        rating: 4,
                        condition: ActionCondition::HpBelow { percent: 50 },
                    },
                ],
                note: String::new(),
            },
        ],
        troops: vec![
            Troop::default(),
            Troop {
                id: 1,
                name: "A Meadow Slime".into(),
                members: vec![TroopMember {
                    enemy_id: 1,
                    x: 220,
                    y: 120,
                }],
                pages: vec![TroopPage {
                    condition: TroopCondition {
                        turn_start: Some(0),
                        turn_every: 0,
                        enemy_hp: None,
                        switch_id: 0,
                    },
                    span: Span::Battle,
                    list: vec![Command::ShowText {
                        face: String::new(),
                        speaker: String::new(),
                        lines: vec!["A meadow slime blocks the path.".into()],
                    }],
                }],
            },
        ],
        states: vec![
            State::default(),
            State {
                id: 1,
                name: "Knockout".into(),
                restriction: Restriction::CannotAct,
                removal: Removal::None,
                slip_hp_percent: 0,
                note: "Applied when HP reaches 0. Healing does not remove it.".into(),
            },
            State {
                id: 2,
                name: "Poison".into(),
                restriction: Restriction::None,
                removal: Removal::Turns { min: 3, max: 3 },
                slip_hp_percent: 10,
                note: String::new(),
            },
        ],
        tilesets: vec![Tileset::default(), meadow_tileset()],
        common_events: vec![
            CommonEvent::default(),
            CommonEvent {
                id: 1,
                name: "Rest".into(),
                trigger: CommonTrigger::None,
                switch_id: 0,
                list: vec![
                    Command::RecoverAll {
                        target: ActorTarget::EntireParty,
                    },
                    Command::ShowText {
                        face: String::new(),
                        speaker: String::new(),
                        lines: vec!["Warmth returns to your limbs.".into()],
                    },
                ],
            },
        ],
        switches: vec![String::new(), "Heard the Rumor".into()],
        variables: vec![String::new(), "Talks".into()],
    };
    game.normalize();
    (game, vec![meadow_map()])
}

fn params_with(index: usize, value: i32) -> Vec<i32> {
    let mut params = vec![0; PARAM_COUNT];
    params[index] = value;
    params
}

fn meadow_tileset() -> Tileset {
    let mut tileset = Tileset::blank(1);
    tileset.name = "Meadow".into();
    tileset.image = "meadow".into();
    tileset.note =
        "Lower layer is ground. Upper layer is flowers and trees. Tile 8 is the path autotile."
            .into();
    tileset
}

fn meadow_map() -> Map {
    let mut map = Map::blank(1, "Hollow", 20, 15, 1, GRASS);
    for x in 0..map.width {
        map.set_upper(x, 0, TREE);
        map.set_upper(x, map.height - 1, TREE);
    }
    for y in 0..map.height {
        map.set_upper(0, y, TREE);
        map.set_upper(map.width - 1, y, TREE);
    }
    for y in 1..=2 {
        for x in 1..=3 {
            map.set_lower(x, y, WATER);
        }
    }
    let path = [
        (3, 12),
        (3, 11),
        (3, 10),
        (3, 9),
        (3, 8),
        (3, 7),
        (3, 6),
        (4, 6),
        (5, 6),
        (6, 6),
        (7, 6),
        (8, 6),
        (9, 6),
        (10, 6),
        (11, 6),
        (12, 6),
        (13, 6),
        (14, 6),
        (14, 7),
        (14, 8),
        (14, 9),
        (14, 10),
        (14, 11),
        (15, 11),
        (16, 11),
        (17, 11),
    ];
    for (x, y) in path {
        map.set_lower(x, y, PATH);
    }
    map.set_lower(15, 8, STONE);
    map.set_lower(16, 8, STONE);
    map.set_upper(6, 8, FLOWER);
    map.set_upper(7, 4, FLOWER);

    map.events.push(Event {
        id: 1,
        name: "Mara".into(),
        x: 8,
        y: 5,
        pages: vec![Page {
            image: EventImage {
                character: "mara".into(),
                index: 0,
                direction: Direction::Down,
                pattern: 1,
            },
            direction_fix: true,
            list: mara_commands(),
            ..Page::default()
        }],
    });
    map.events.push(Event {
        id: 2,
        name: "Board".into(),
        x: 5,
        y: 5,
        pages: vec![Page {
            image: EventImage {
                character: "board".into(),
                index: 0,
                direction: Direction::Down,
                pattern: 1,
            },
            direction_fix: true,
            list: vec![
                Command::ShowPicture {
                    id: 1,
                    name: "board".into(),
                    x: 168,
                    y: 36,
                },
                Command::Wait { frames: 12 },
                Command::ShowText {
                    face: String::new(),
                    speaker: String::new(),
                    lines: vec![
                        "The board reads: the path runs east,".into(),
                        "then south. Mind the slime.".into(),
                    ],
                },
                Command::ErasePicture { id: 1 },
            ],
            ..Page::default()
        }],
    });
    map.events.push(Event {
        id: 3,
        name: "Chest".into(),
        x: 15,
        y: 9,
        pages: vec![
            Page {
                image: EventImage {
                    character: "chest".into(),
                    index: 0,
                    direction: Direction::Down,
                    pattern: 1,
                },
                direction_fix: true,
                list: vec![
                    Command::Comment {
                        text: "One berry, and only once.".into(),
                    },
                    Command::ShowText {
                        face: String::new(),
                        speaker: String::new(),
                        lines: vec!["A wooden chest. The latch is loose.".into()],
                    },
                    Command::ChangeItems {
                        id: 2,
                        op: AmountOp::Gain,
                        amount: 1,
                    },
                    Command::ShowText {
                        face: String::new(),
                        speaker: String::new(),
                        lines: vec!["Received a Berry!".into()],
                    },
                    Command::ControlSelfSwitch {
                        key: SelfSwitch::A,
                        on: true,
                    },
                ],
                ..Page::default()
            },
            Page {
                conditions: PageConditions {
                    self_switch: Some(SelfSwitch::A),
                    ..PageConditions::default()
                },
                image: EventImage {
                    character: "chest".into(),
                    index: 1,
                    direction: Direction::Down,
                    pattern: 1,
                },
                direction_fix: true,
                list: vec![Command::ShowText {
                    face: String::new(),
                    speaker: String::new(),
                    lines: vec!["The chest is empty.".into()],
                }],
                ..Page::default()
            },
        ],
    });
    map.events.push(Event {
        id: 4,
        name: "Slime".into(),
        x: 18,
        y: 11,
        pages: vec![
            Page {
                image: EventImage {
                    character: "slime".into(),
                    index: 0,
                    direction: Direction::Down,
                    pattern: 1,
                },
                trigger: Trigger::PlayerTouch,
                list: vec![Command::Battle {
                    troop_id: 1,
                    can_escape: true,
                    on_win: vec![Command::ControlSelfSwitch {
                        key: SelfSwitch::A,
                        on: true,
                    }],
                    on_escape: vec![Command::ShowText {
                        face: String::new(),
                        speaker: String::new(),
                        lines: vec!["The slime looks almost disappointed.".into()],
                    }],
                }],
                ..Page::default()
            },
            Page {
                conditions: PageConditions {
                    self_switch: Some(SelfSwitch::A),
                    ..PageConditions::default()
                },
                image: EventImage::default(),
                trigger: Trigger::Action,
                priority: Priority::Below,
                through: true,
                list: Vec::new(),
                ..Page::default()
            },
        ],
    });
    map
}

fn mara_commands() -> Vec<Command> {
    vec![
        Command::ShowText {
            face: "mara".into(),
            speaker: "Mara".into(),
            lines: vec![
                "The meadow is quiet, but the east".into(),
                "path is not. A slime has been sitting".into(),
                "on it all morning.".into(),
            ],
        },
        Command::ShowChoices {
            choices: vec![
                "Where is the chest?".into(),
                "I could use a rest.".into(),
                "Goodbye.".into(),
            ],
            cancel: CancelMode::Choice(2),
            branches: vec![
                vec![
                    Command::ShowText {
                        face: "mara".into(),
                        speaker: "Mara".into(),
                        lines: vec![
                            "Beside the gray stones, just south".into(),
                            "of the long path.".into(),
                        ],
                    },
                    Command::ControlSwitch {
                        id: 1,
                        value: SwitchValue::On,
                    },
                    Command::ControlVariable {
                        id: 1,
                        op: VarOp::Add,
                        operand: Operand::Constant { value: 1 },
                    },
                ],
                vec![Command::CallCommon { id: 1 }],
                vec![Command::ShowText {
                    face: "mara".into(),
                    speaker: "Mara".into(),
                    lines: vec!["Safe travels.".into()],
                }],
            ],
            cancel_branch: Vec::new(),
        },
    ]
}

/// A second map used by the transfer test and by anyone who adds one.
pub fn clearing_map() -> Map {
    let mut map = Map::blank(2, "Clearing", 17, 13, 1, GRASS);
    for x in 0..map.width {
        map.set_upper(x, 0, TREE);
        map.set_upper(x, map.height - 1, TREE);
    }
    for y in 0..map.height {
        map.set_upper(0, y, TREE);
        map.set_upper(map.width - 1, y, TREE);
    }
    map
}
