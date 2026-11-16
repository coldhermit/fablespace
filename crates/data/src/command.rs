//! Event commands and the tree the editor walks.

use serde::{Deserialize, Serialize};

use crate::model::{Direction, SelfSwitch};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum AmountOp {
    #[default]
    Gain,
    Lose,
}

impl std::fmt::Display for AmountOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            AmountOp::Gain => "Gain",
            AmountOp::Lose => "Lose",
        })
    }
}

impl AmountOp {
    pub fn all() -> [AmountOp; 2] {
        [AmountOp::Gain, AmountOp::Lose]
    }

    pub fn apply(self, current: i32, amount: i32) -> i32 {
        let amount = amount.abs();
        match self {
            AmountOp::Gain => current.saturating_add(amount),
            AmountOp::Lose => current.saturating_sub(amount).max(0),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ActorTarget {
    EntireParty,
    Actor(u32),
}

impl Default for ActorTarget {
    fn default() -> Self {
        ActorTarget::EntireParty
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum VarOp {
    #[default]
    Set,
    Add,
    Sub,
    Mul,
    Div,
    Mod,
}

impl std::fmt::Display for VarOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            VarOp::Set => "Set",
            VarOp::Add => "Add",
            VarOp::Sub => "Sub",
            VarOp::Mul => "Mul",
            VarOp::Div => "Div",
            VarOp::Mod => "Mod",
        })
    }
}

impl VarOp {
    pub fn all() -> [VarOp; 6] {
        [
            VarOp::Set,
            VarOp::Add,
            VarOp::Sub,
            VarOp::Mul,
            VarOp::Div,
            VarOp::Mod,
        ]
    }

    pub fn apply(self, current: i32, value: i32) -> i32 {
        match self {
            VarOp::Set => value,
            VarOp::Add => current.saturating_add(value),
            VarOp::Sub => current.saturating_sub(value),
            VarOp::Mul => current.saturating_mul(value),
            VarOp::Div => {
                if value == 0 {
                    current
                } else {
                    current / value
                }
            }
            VarOp::Mod => {
                if value == 0 {
                    current
                } else {
                    current % value
                }
            }
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum SwitchValue {
    #[default]
    On,
    Off,
    Toggle,
}

impl std::fmt::Display for SwitchValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            SwitchValue::On => "ON",
            SwitchValue::Off => "OFF",
            SwitchValue::Toggle => "Toggle",
        })
    }
}

impl SwitchValue {
    pub fn all() -> [SwitchValue; 3] {
        [SwitchValue::On, SwitchValue::Off, SwitchValue::Toggle]
    }

    pub fn apply(self, current: bool) -> bool {
        match self {
            SwitchValue::On => true,
            SwitchValue::Off => false,
            SwitchValue::Toggle => !current,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum Cmp {
    #[default]
    Eq,
    Ne,
    Lt,
    Gt,
    Le,
    Ge,
}

impl std::fmt::Display for Cmp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Cmp::Eq => "==",
            Cmp::Ne => "!=",
            Cmp::Lt => "<",
            Cmp::Gt => ">",
            Cmp::Le => "<=",
            Cmp::Ge => ">=",
        })
    }
}

impl Cmp {
    pub fn all() -> [Cmp; 6] {
        [Cmp::Eq, Cmp::Ne, Cmp::Lt, Cmp::Gt, Cmp::Le, Cmp::Ge]
    }

    pub fn test(self, left: i32, right: i32) -> bool {
        match self {
            Cmp::Eq => left == right,
            Cmp::Ne => left != right,
            Cmp::Lt => left < right,
            Cmp::Gt => left > right,
            Cmp::Le => left <= right,
            Cmp::Ge => left >= right,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Operand {
    Constant { value: i32 },
    Variable { id: u32 },
    Random { min: i32, max: i32 },
    Gold,
    ItemCount { id: u32 },
    ActorHp { id: u32 },
    ActorMp { id: u32 },
    ActorLevel { id: u32 },
}

impl Default for Operand {
    fn default() -> Self {
        Operand::Constant { value: 0 }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Condition {
    Switch { id: u32, on: bool },
    Variable { id: u32, cmp: Cmp, operand: Operand },
    SelfSwitch { key: SelfSwitch, on: bool },
    Item { id: u32 },
    Gold { cmp: Cmp, amount: i32 },
    ActorInParty { id: u32 },
    ActorHp { id: u32, cmp: Cmp, amount: i32 },
}

impl Default for Condition {
    fn default() -> Self {
        Condition::Switch { id: 1, on: true }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum CancelMode {
    #[default]
    Disallow,
    Choice(usize),
    Branch,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum MoveTarget {
    Player,
    ThisEvent,
    Event { id: u32 },
}

impl Default for MoveTarget {
    fn default() -> Self {
        MoveTarget::ThisEvent
    }
}

impl std::fmt::Display for MoveTarget {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MoveTarget::Player => f.write_str("Player"),
            MoveTarget::ThisEvent => f.write_str("This Event"),
            MoveTarget::Event { id } => write!(f, "Event {id}"),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum MoveStep {
    Down,
    Left,
    Right,
    Up,
    TowardPlayer,
    AwayFromPlayer,
    Forward,
    TurnDown,
    TurnLeft,
    TurnRight,
    TurnUp,
    TurnTowardPlayer,
    Wait { frames: u32 },
    Speed { speed: u8 },
    Through { on: bool },
}

impl Default for MoveStep {
    fn default() -> Self {
        MoveStep::Down
    }
}

impl std::fmt::Display for MoveStep {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MoveStep::Down => f.write_str("Move Down"),
            MoveStep::Left => f.write_str("Move Left"),
            MoveStep::Right => f.write_str("Move Right"),
            MoveStep::Up => f.write_str("Move Up"),
            MoveStep::TowardPlayer => f.write_str("Move toward Player"),
            MoveStep::AwayFromPlayer => f.write_str("Move away from Player"),
            MoveStep::Forward => f.write_str("One Step Forward"),
            MoveStep::TurnDown => f.write_str("Turn Down"),
            MoveStep::TurnLeft => f.write_str("Turn Left"),
            MoveStep::TurnRight => f.write_str("Turn Right"),
            MoveStep::TurnUp => f.write_str("Turn Up"),
            MoveStep::TurnTowardPlayer => f.write_str("Turn toward Player"),
            MoveStep::Wait { frames } => write!(f, "Wait {frames}"),
            MoveStep::Speed { speed } => write!(f, "Speed {speed}"),
            MoveStep::Through { on } => write!(f, "Through {}", if *on { "ON" } else { "OFF" }),
        }
    }
}

impl MoveStep {
    pub fn catalog() -> Vec<MoveStep> {
        vec![
            MoveStep::Down,
            MoveStep::Left,
            MoveStep::Right,
            MoveStep::Up,
            MoveStep::TowardPlayer,
            MoveStep::AwayFromPlayer,
            MoveStep::Forward,
            MoveStep::TurnDown,
            MoveStep::TurnLeft,
            MoveStep::TurnRight,
            MoveStep::TurnUp,
            MoveStep::TurnTowardPlayer,
            MoveStep::Wait { frames: 30 },
            MoveStep::Speed { speed: 4 },
            MoveStep::Through { on: true },
            MoveStep::Through { on: false },
        ]
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MoveRoute {
    pub steps: Vec<MoveStep>,
    pub repeat: bool,
    pub skippable: bool,
    pub wait: bool,
}

impl Default for MoveRoute {
    fn default() -> Self {
        Self {
            steps: Vec::new(),
            repeat: false,
            skippable: true,
            wait: true,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "code", rename_all = "snake_case")]
pub enum Command {
    ShowText {
        face: String,
        speaker: String,
        lines: Vec<String>,
    },
    ShowChoices {
        choices: Vec<String>,
        cancel: CancelMode,
        branches: Vec<Vec<Command>>,
        cancel_branch: Vec<Command>,
    },
    Comment {
        text: String,
    },
    ControlSwitch {
        id: u32,
        value: SwitchValue,
    },
    ControlVariable {
        id: u32,
        op: VarOp,
        operand: Operand,
    },
    ControlSelfSwitch {
        key: SelfSwitch,
        on: bool,
    },
    Conditional {
        cond: Condition,
        then_branch: Vec<Command>,
        else_branch: Option<Vec<Command>>,
    },
    CallCommon {
        id: u32,
    },
    ExitEvent,
    Wait {
        frames: u32,
    },
    TransferPlayer {
        map_id: u32,
        x: u32,
        y: u32,
        direction: Option<Direction>,
        fade: bool,
    },
    SetMoveRoute {
        target: MoveTarget,
        route: MoveRoute,
    },
    ChangeGold {
        op: AmountOp,
        amount: i32,
    },
    ChangeItems {
        id: u32,
        op: AmountOp,
        amount: i32,
    },
    ChangeHp {
        target: ActorTarget,
        op: AmountOp,
        amount: i32,
    },
    ChangeMp {
        target: ActorTarget,
        op: AmountOp,
        amount: i32,
    },
    RecoverAll {
        target: ActorTarget,
    },
    GameOver,
    OpenSave,
    OpenMenu,
    Battle {
        troop_id: u32,
        can_escape: bool,
        on_win: Vec<Command>,
        on_escape: Vec<Command>,
    },
    ReturnToTitle,
    ShowPicture {
        id: u32,
        name: String,
        x: i32,
        y: i32,
    },
    MovePicture {
        id: u32,
        x: i32,
        y: i32,
        frames: u32,
    },
    ErasePicture {
        id: u32,
    },
}

impl Command {
    pub fn blank(kind: &str) -> Self {
        match kind {
            "show_text" => Command::ShowText {
                face: String::new(),
                speaker: String::new(),
                lines: vec!["...".into()],
            },
            "show_choices" => Command::ShowChoices {
                choices: vec!["Yes".into(), "No".into()],
                cancel: CancelMode::Choice(1),
                branches: vec![Vec::new(), Vec::new()],
                cancel_branch: Vec::new(),
            },
            "comment" => Command::Comment {
                text: String::new(),
            },
            "control_switch" => Command::ControlSwitch {
                id: 1,
                value: SwitchValue::On,
            },
            "control_variable" => Command::ControlVariable {
                id: 1,
                op: VarOp::Set,
                operand: Operand::Constant { value: 0 },
            },
            "control_self_switch" => Command::ControlSelfSwitch {
                key: SelfSwitch::A,
                on: true,
            },
            "conditional" => Command::Conditional {
                cond: Condition::default(),
                then_branch: Vec::new(),
                else_branch: None,
            },
            "call_common" => Command::CallCommon { id: 1 },
            "exit_event" => Command::ExitEvent,
            "wait" => Command::Wait { frames: 30 },
            "transfer_player" => Command::TransferPlayer {
                map_id: 1,
                x: 0,
                y: 0,
                direction: None,
                fade: true,
            },
            "set_move_route" => Command::SetMoveRoute {
                target: MoveTarget::ThisEvent,
                route: MoveRoute {
                    steps: vec![MoveStep::Down],
                    ..MoveRoute::default()
                },
            },
            "change_gold" => Command::ChangeGold {
                op: AmountOp::Gain,
                amount: 10,
            },
            "change_items" => Command::ChangeItems {
                id: 1,
                op: AmountOp::Gain,
                amount: 1,
            },
            "change_hp" => Command::ChangeHp {
                target: ActorTarget::EntireParty,
                op: AmountOp::Gain,
                amount: 10,
            },
            "change_mp" => Command::ChangeMp {
                target: ActorTarget::EntireParty,
                op: AmountOp::Gain,
                amount: 5,
            },
            "recover_all" => Command::RecoverAll {
                target: ActorTarget::EntireParty,
            },
            "game_over" => Command::GameOver,
            "open_save" => Command::OpenSave,
            "open_menu" => Command::OpenMenu,
            "battle" => Command::Battle {
                troop_id: 1,
                can_escape: true,
                on_win: Vec::new(),
                on_escape: Vec::new(),
            },
            "return_to_title" => Command::ReturnToTitle,
            "show_picture" => Command::ShowPicture {
                id: 1,
                name: String::new(),
                x: 160,
                y: 48,
            },
            "move_picture" => Command::MovePicture {
                id: 1,
                x: 160,
                y: 48,
                frames: 30,
            },
            "erase_picture" => Command::ErasePicture { id: 1 },
            _ => Command::Comment {
                text: String::new(),
            },
        }
    }

    pub fn kind_name(&self) -> &'static str {
        match self {
            Command::ShowText { .. } => "show_text",
            Command::ShowChoices { .. } => "show_choices",
            Command::Comment { .. } => "comment",
            Command::ControlSwitch { .. } => "control_switch",
            Command::ControlVariable { .. } => "control_variable",
            Command::ControlSelfSwitch { .. } => "control_self_switch",
            Command::Conditional { .. } => "conditional",
            Command::CallCommon { .. } => "call_common",
            Command::ExitEvent => "exit_event",
            Command::Wait { .. } => "wait",
            Command::TransferPlayer { .. } => "transfer_player",
            Command::SetMoveRoute { .. } => "set_move_route",
            Command::ChangeGold { .. } => "change_gold",
            Command::ChangeItems { .. } => "change_items",
            Command::ChangeHp { .. } => "change_hp",
            Command::ChangeMp { .. } => "change_mp",
            Command::RecoverAll { .. } => "recover_all",
            Command::GameOver => "game_over",
            Command::OpenSave => "open_save",
            Command::OpenMenu => "open_menu",
            Command::Battle { .. } => "battle",
            Command::ReturnToTitle => "return_to_title",
            Command::ShowPicture { .. } => "show_picture",
            Command::MovePicture { .. } => "move_picture",
            Command::ErasePicture { .. } => "erase_picture",
        }
    }

    pub fn kinds() -> &'static [(&'static str, &'static str)] {
        &[
            ("show_text", "Show Text"),
            ("show_choices", "Show Choices"),
            ("comment", "Comment"),
            ("control_switch", "Control Switches"),
            ("control_variable", "Control Variables"),
            ("control_self_switch", "Control Self Switch"),
            ("conditional", "Conditional Branch"),
            ("call_common", "Common Event"),
            ("exit_event", "Exit Event Processing"),
            ("wait", "Wait"),
            ("transfer_player", "Transfer Player"),
            ("set_move_route", "Set Move Route"),
            ("change_gold", "Change Gold"),
            ("change_items", "Change Items"),
            ("change_hp", "Change HP"),
            ("change_mp", "Change MP"),
            ("recover_all", "Recover All"),
            ("game_over", "Game Over"),
            ("open_save", "Open Save Screen"),
            ("open_menu", "Open Menu Screen"),
            ("battle", "Battle Processing"),
            ("return_to_title", "Return to Title"),
            ("show_picture", "Show Picture"),
            ("move_picture", "Move Picture"),
            ("erase_picture", "Erase Picture"),
        ]
    }
}

pub fn summary(cmd: &Command) -> String {
    match cmd {
        Command::ShowText { speaker, lines, .. } => {
            let line = lines
                .iter()
                .find(|l| !l.is_empty())
                .map(|s| s.as_str())
                .unwrap_or("");
            if speaker.is_empty() {
                format!("Show Text: {line}")
            } else {
                format!("Show Text: {speaker}: {line}")
            }
        }
        Command::ShowChoices { choices, .. } => format!("Show Choices: {}", choices.join(" / ")),
        Command::Comment { text } => format!("Comment: {text}"),
        Command::ControlSwitch { id, value } => format!("Control Switch [{id:04}] = {value}"),
        Command::ControlVariable { id, op, .. } => format!("Control Variable [{id:04}] {op}"),
        Command::ControlSelfSwitch { key, on } => {
            format!("Self Switch {key} = {}", if *on { "ON" } else { "OFF" })
        }
        Command::Conditional { .. } => "Conditional Branch".into(),
        Command::CallCommon { id } => format!("Common Event [{id:04}]"),
        Command::ExitEvent => "Exit Event Processing".into(),
        Command::Wait { frames } => format!("Wait {frames} frames"),
        Command::TransferPlayer { map_id, x, y, .. } => {
            format!("Transfer Player to map {map_id} ({x}, {y})")
        }
        Command::SetMoveRoute { target, .. } => format!("Set Move Route: {target}"),
        Command::ChangeGold { op, amount } => format!("Change Gold: {op} {amount}"),
        Command::ChangeItems { id, op, amount } => format!("Change Items [{id:04}]: {op} {amount}"),
        Command::ChangeHp { op, amount, .. } => format!("Change HP: {op} {amount}"),
        Command::ChangeMp { op, amount, .. } => format!("Change MP: {op} {amount}"),
        Command::RecoverAll { .. } => "Recover All".into(),
        Command::GameOver => "Game Over".into(),
        Command::OpenSave => "Open Save Screen".into(),
        Command::OpenMenu => "Open Menu Screen".into(),
        Command::Battle { troop_id, .. } => format!("Battle Processing [{troop_id:04}]"),
        Command::ReturnToTitle => "Return to Title".into(),
        Command::ShowPicture { id, name, .. } => format!("Show Picture {id}: {name}"),
        Command::MovePicture { id, x, y, frames } => {
            format!("Move Picture {id} to ({x}, {y}) in {frames}")
        }
        Command::ErasePicture { id } => format!("Erase Picture {id}"),
    }
}

/// A step in a command path. `Cmd` selects a command; the branch steps enter it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Step {
    Cmd(usize),
    Then,
    Else,
    Choice(usize),
    Cancel,
    Win,
    Escape,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Slot {
    Then,
    Else,
    Choice(usize),
    Cancel,
    Win,
    Escape,
}

impl Slot {
    fn step(self) -> Step {
        match self {
            Slot::Then => Step::Then,
            Slot::Else => Step::Else,
            Slot::Choice(i) => Step::Choice(i),
            Slot::Cancel => Step::Cancel,
            Slot::Win => Step::Win,
            Slot::Escape => Step::Escape,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Target {
    Command(Vec<Step>),
    Branch { parent: Vec<Step>, slot: Slot },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Row {
    pub depth: u32,
    pub label: String,
    pub target: Target,
}

pub fn rows(list: &[Command]) -> Vec<Row> {
    let mut out = Vec::new();
    flatten(list, &[], 0, &mut out);
    out
}

fn flatten(list: &[Command], prefix: &[Step], depth: u32, out: &mut Vec<Row>) {
    for (i, cmd) in list.iter().enumerate() {
        let mut path = prefix.to_vec();
        path.push(Step::Cmd(i));
        out.push(Row {
            depth,
            label: summary(cmd),
            target: Target::Command(path.clone()),
        });
        match cmd {
            Command::ShowChoices {
                choices,
                branches,
                cancel,
                cancel_branch,
            } => {
                for (ci, choice) in choices.iter().enumerate() {
                    out.push(Row {
                        depth: depth + 1,
                        label: format!("When \"{choice}\""),
                        target: Target::Branch {
                            parent: path.clone(),
                            slot: Slot::Choice(ci),
                        },
                    });
                    let child = branches.get(ci).map(Vec::as_slice).unwrap_or(&[]);
                    let mut child_prefix = path.clone();
                    child_prefix.push(Step::Choice(ci));
                    flatten(child, &child_prefix, depth + 2, out);
                }
                if matches!(cancel, CancelMode::Branch) {
                    out.push(Row {
                        depth: depth + 1,
                        label: "When Cancel".into(),
                        target: Target::Branch {
                            parent: path.clone(),
                            slot: Slot::Cancel,
                        },
                    });
                    let mut child_prefix = path.clone();
                    child_prefix.push(Step::Cancel);
                    flatten(cancel_branch, &child_prefix, depth + 2, out);
                }
            }
            Command::Conditional {
                then_branch,
                else_branch,
                ..
            } => {
                out.push(Row {
                    depth: depth + 1,
                    label: "Branch".into(),
                    target: Target::Branch {
                        parent: path.clone(),
                        slot: Slot::Then,
                    },
                });
                let mut then_prefix = path.clone();
                then_prefix.push(Step::Then);
                flatten(then_branch, &then_prefix, depth + 2, out);
                if else_branch.is_some() {
                    out.push(Row {
                        depth: depth + 1,
                        label: "Else".into(),
                        target: Target::Branch {
                            parent: path.clone(),
                            slot: Slot::Else,
                        },
                    });
                    let mut else_prefix = path.clone();
                    else_prefix.push(Step::Else);
                    flatten(
                        else_branch.as_deref().unwrap_or(&[]),
                        &else_prefix,
                        depth + 2,
                        out,
                    );
                }
            }
            Command::Battle {
                on_win, on_escape, ..
            } => {
                out.push(Row {
                    depth: depth + 1,
                    label: "If Win".into(),
                    target: Target::Branch {
                        parent: path.clone(),
                        slot: Slot::Win,
                    },
                });
                let mut win_prefix = path.clone();
                win_prefix.push(Step::Win);
                flatten(on_win, &win_prefix, depth + 2, out);
                out.push(Row {
                    depth: depth + 1,
                    label: "If Escape".into(),
                    target: Target::Branch {
                        parent: path.clone(),
                        slot: Slot::Escape,
                    },
                });
                let mut esc_prefix = path.clone();
                esc_prefix.push(Step::Escape);
                flatten(on_escape, &esc_prefix, depth + 2, out);
            }
            _ => {}
        }
    }
}

fn branch_mut<'a>(cmd: &'a mut Command, slot: Slot) -> Option<&'a mut Vec<Command>> {
    match (cmd, slot) {
        (Command::Conditional { then_branch, .. }, Slot::Then) => Some(then_branch),
        (Command::Conditional { else_branch, .. }, Slot::Else) => else_branch.as_mut(),
        (Command::ShowChoices { branches, .. }, Slot::Choice(i)) => branches.get_mut(i),
        (Command::ShowChoices { cancel_branch, .. }, Slot::Cancel) => Some(cancel_branch),
        (Command::Battle { on_win, .. }, Slot::Win) => Some(on_win),
        (Command::Battle { on_escape, .. }, Slot::Escape) => Some(on_escape),
        _ => None,
    }
}

fn list_at<'a>(root: &'a mut Vec<Command>, prefix: &[Step]) -> Option<&'a mut Vec<Command>> {
    if prefix.is_empty() {
        return Some(root);
    }
    let (head, rest) = prefix.split_first()?;
    let Step::Cmd(index) = *head else {
        return None;
    };
    if rest.is_empty() {
        return None;
    }
    let (slot_step, tail) = rest.split_first()?;
    let slot = match *slot_step {
        Step::Then => Slot::Then,
        Step::Else => Slot::Else,
        Step::Choice(i) => Slot::Choice(i),
        Step::Cancel => Slot::Cancel,
        Step::Win => Slot::Win,
        Step::Escape => Slot::Escape,
        Step::Cmd(_) => return None,
    };
    let cmd = root.get_mut(index)?;
    let child = branch_mut(cmd, slot)?;
    list_at(child, tail)
}

pub fn command_mut<'a>(root: &'a mut Vec<Command>, path: &[Step]) -> Option<&'a mut Command> {
    let (last, parent) = path.split_last()?;
    let Step::Cmd(index) = *last else {
        return None;
    };
    let list = if parent.is_empty() {
        root
    } else {
        list_at(root, parent)?
    };
    list.get_mut(index)
}

pub fn insert_after(root: &mut Vec<Command>, path: &[Step], cmd: Command) -> bool {
    let Some(Step::Cmd(index)) = path.last().copied() else {
        return false;
    };
    let parent = &path[..path.len() - 1];
    let Some(list) = (if parent.is_empty() {
        Some(root)
    } else {
        list_at(root, parent)
    }) else {
        return false;
    };
    let at = (index + 1).min(list.len());
    list.insert(at, cmd);
    true
}

pub fn insert_in_branch(
    root: &mut Vec<Command>,
    parent: &[Step],
    slot: Slot,
    cmd: Command,
) -> bool {
    let Some(list) = ({
        let mut prefix = parent.to_vec();
        prefix.push(slot.step());
        list_at(root, &prefix)
    }) else {
        return false;
    };
    list.push(cmd);
    true
}

pub fn remove_command(root: &mut Vec<Command>, path: &[Step]) -> bool {
    let Some(Step::Cmd(index)) = path.last().copied() else {
        return false;
    };
    let parent = &path[..path.len() - 1];
    let Some(list) = (if parent.is_empty() {
        Some(root)
    } else {
        list_at(root, parent)
    }) else {
        return false;
    };
    if index < list.len() {
        list.remove(index);
        true
    } else {
        false
    }
}

pub fn move_command(root: &mut Vec<Command>, path: &[Step], delta: isize) -> bool {
    let Some(Step::Cmd(index)) = path.last().copied() else {
        return false;
    };
    let parent = &path[..path.len() - 1];
    let Some(list) = (if parent.is_empty() {
        Some(root)
    } else {
        list_at(root, parent)
    }) else {
        return false;
    };
    let next = index as isize + delta;
    if next < 0 || next >= list.len() as isize {
        return false;
    }
    list.swap(index, next as usize);
    true
}

/// Keep choice branches the same length as the choice labels.
pub fn sync_choice_branches(cmd: &mut Command) {
    if let Command::ShowChoices {
        choices, branches, ..
    } = cmd
    {
        branches.resize_with(choices.len(), Vec::new);
    }
}

pub fn set_else(cmd: &mut Command, enabled: bool) {
    if let Command::Conditional { else_branch, .. } = cmd {
        if enabled && else_branch.is_none() {
            *else_branch = Some(Vec::new());
        }
        if !enabled {
            *else_branch = None;
        }
    }
}
