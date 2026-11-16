//! The event interpreter. Instant commands run in a burst until a wait.

use fablespace_data::{ActorTarget, AmountOp, Command, Condition, MoveTarget, Operand, TILE};

use crate::party::StateInst;
use crate::session::{Frame, Interp, Process, Session, Wait};

pub const STEP_CAP: usize = 8_000;
pub const CALL_CAP: usize = 16;

impl Interp {
    pub fn new(commands: Vec<Command>) -> Self {
        Self {
            stack: vec![Frame { commands, ip: 0 }],
            wait: Wait::None,
            map_id: 0,
            event_id: None,
        }
    }

    pub fn finished(&self) -> bool {
        self.stack.is_empty() && matches!(self.wait, Wait::None)
    }
}

impl Session {
    pub(crate) fn run_interp(&mut self, interp: &mut Interp) {
        let mut steps = 0;
        loop {
            if steps > STEP_CAP {
                if matches!(interp.wait, Wait::None) {
                    interp.wait = Wait::Frames(1);
                }
                break;
            }
            if let Wait::Frames(n) = interp.wait {
                if n > 1 {
                    interp.wait = Wait::Frames(n - 1);
                    break;
                }
                interp.wait = Wait::None;
            }
            let route_target = match &interp.wait {
                Wait::Route(target) => Some(*target),
                _ => None,
            };
            if let Some(target) = route_target {
                if !self.route_running(target, interp) {
                    interp.wait = Wait::None;
                }
            }
            if !matches!(interp.wait, Wait::None) {
                break;
            }
            let Some(frame) = interp.stack.last_mut() else {
                break;
            };
            if frame.ip >= frame.commands.len() {
                interp.stack.pop();
                continue;
            }
            let cmd = frame.commands[frame.ip].clone();
            frame.ip += 1;
            steps += 1;
            if self.exec(interp, cmd) {
                break;
            }
        }
    }

    /// Returns true when the burst should stop.
    fn exec(&mut self, interp: &mut Interp, cmd: Command) -> bool {
        match cmd {
            Command::ShowText {
                face,
                speaker,
                lines,
            } => {
                self.show_lines(speaker, face, lines);
                interp.wait = Wait::Message;
                true
            }
            Command::ShowChoices {
                choices,
                cancel,
                branches,
                cancel_branch,
            } => {
                if choices.is_empty() {
                    return false;
                }
                self.choice = Some(crate::session::ChoiceUi {
                    options: choices,
                    cursor: 0,
                    branches,
                    cancel,
                    cancel_branch,
                });
                interp.wait = Wait::Choice;
                true
            }
            Command::Comment { .. } => false,
            Command::ControlSwitch { id, value } => {
                self.set_switch(id, value);
                self.refresh_event_pages();
                false
            }
            Command::ControlVariable { id, op, operand } => {
                let value = self.eval_operand(&operand);
                let current = self.variable(id);
                self.set_variable(id, op.apply(current, value));
                self.refresh_event_pages();
                false
            }
            Command::ControlSelfSwitch { key, on } => {
                if let Some(event_id) = interp.event_id {
                    self.set_self_switch(interp.map_id, event_id, key, on);
                    self.refresh_event_pages();
                }
                false
            }
            Command::Conditional {
                cond,
                then_branch,
                else_branch,
            } => {
                let branch = if self.eval_condition(&cond, interp) {
                    then_branch
                } else {
                    else_branch.unwrap_or_default()
                };
                if !branch.is_empty() {
                    interp.stack.push(Frame {
                        commands: branch,
                        ip: 0,
                    });
                }
                false
            }
            Command::CallCommon { id } => {
                if interp.stack.len() >= CALL_CAP {
                    return false;
                }
                if let Some(event) = self.game.common_event(id) {
                    if !event.list.is_empty() {
                        interp.stack.push(Frame {
                            commands: event.list.clone(),
                            ip: 0,
                        });
                    }
                }
                false
            }
            Command::ExitEvent => {
                interp.stack.clear();
                interp.wait = Wait::None;
                true
            }
            Command::Wait { frames } => {
                interp.wait = Wait::Frames(frames.max(1));
                true
            }
            Command::TransferPlayer {
                map_id,
                x,
                y,
                direction,
                fade,
            } => {
                self.transfer(map_id, x, y, direction);
                if fade {
                    self.fade = 14;
                }
                // The running interpreter keeps going on the new map.
                interp.map_id = self.map_id;
                false
            }
            Command::SetMoveRoute { target, route } => {
                let resolved = self.resolve_target(target, interp);
                self.install_route(resolved, route.clone());
                if route.wait {
                    interp.wait = Wait::Route(resolved);
                    true
                } else {
                    false
                }
            }
            Command::ChangeGold { op, amount } => {
                self.gold = op.apply(self.gold, amount);
                false
            }
            Command::ChangeItems { id, op, amount } => {
                self.change_items(id, op, amount);
                self.refresh_event_pages();
                false
            }
            Command::ChangeHp { target, op, amount } => {
                self.change_hp(target, op, amount);
                false
            }
            Command::ChangeMp { target, op, amount } => {
                self.change_mp(target, op, amount);
                false
            }
            Command::RecoverAll { target } => {
                self.recover(target);
                false
            }
            Command::GameOver => {
                self.enter_game_over();
                interp.stack.clear();
                true
            }
            Command::OpenSave => {
                self.open_save(false);
                interp.wait = Wait::Save;
                true
            }
            Command::OpenMenu => {
                self.open_menu();
                interp.wait = Wait::Menu;
                true
            }
            Command::Battle {
                troop_id,
                can_escape,
                on_win,
                on_escape,
            } => {
                if self.battle.is_some() {
                    return false;
                }
                if self.game.troop(troop_id).is_none() {
                    return false;
                }
                self.start_battle(troop_id, can_escape);
                interp.wait = Wait::Battle { on_win, on_escape };
                true
            }
            Command::ReturnToTitle => {
                self.to_title();
                interp.stack.clear();
                true
            }
            Command::ShowPicture { id, name, x, y } => {
                self.show_picture(id, name, x, y);
                false
            }
            Command::MovePicture { id, x, y, frames } => {
                self.move_picture(id, x, y, frames);
                false
            }
            Command::ErasePicture { id } => {
                self.pictures.retain(|picture| picture.id != id);
                false
            }
        }
    }

    fn eval_condition(&mut self, cond: &Condition, interp: &Interp) -> bool {
        match cond {
            Condition::Switch { id, on } => self.switch(*id) == *on,
            Condition::Variable { id, cmp, operand } => {
                cmp.test(self.variable(*id), self.eval_operand(operand))
            }
            Condition::SelfSwitch { key, on } => interp
                .event_id
                .is_some_and(|event_id| self.self_switch(interp.map_id, event_id, *key) == *on),
            Condition::Item { id } => self.bag.count_item(*id) > 0,
            Condition::Gold { cmp, amount } => cmp.test(self.gold, *amount),
            Condition::ActorInParty { id } => {
                self.party.iter().any(|member| member.actor_id == *id)
            }
            Condition::ActorHp { id, cmp, amount } => self
                .party
                .iter()
                .find(|member| member.actor_id == *id)
                .is_some_and(|member| cmp.test(member.hp, *amount)),
        }
    }

    fn eval_operand(&mut self, operand: &Operand) -> i32 {
        match operand {
            Operand::Constant { value } => *value,
            Operand::Variable { id } => self.variable(*id),
            Operand::Random { min, max } => self.rng.range(*min, *max),
            Operand::Gold => self.gold,
            Operand::ItemCount { id } => self.bag.count_item(*id) as i32,
            Operand::ActorHp { id } => self.actor_member(*id).map(|m| m.hp).unwrap_or(0),
            Operand::ActorMp { id } => self.actor_member(*id).map(|m| m.mp).unwrap_or(0),
            Operand::ActorLevel { id } => {
                self.actor_member(*id).map(|m| m.level as i32).unwrap_or(0)
            }
        }
    }

    pub(crate) fn target_ids(&self, target: ActorTarget) -> Vec<usize> {
        match target {
            ActorTarget::EntireParty => (0..self.party.len()).collect(),
            ActorTarget::Actor(id) => self
                .party
                .iter()
                .enumerate()
                .find(|(_, member)| member.actor_id == id)
                .map(|(index, _)| index)
                .into_iter()
                .collect(),
        }
    }

    pub(crate) fn change_hp(&mut self, target: ActorTarget, op: AmountOp, amount: i32) {
        let ids = self.target_ids(target);
        let knockout = self.knockout_state();
        for index in ids {
            let max_hp = crate::party::stats_of(&self.game, &self.party[index]).mhp;
            let hp = &mut self.party[index].hp;
            *hp = match op {
                AmountOp::Gain => hp.saturating_add(amount.abs()).min(max_hp),
                AmountOp::Lose => hp.saturating_sub(amount.abs()).max(0),
            };
            if self.party[index].hp == 0 && knockout != 0 {
                add_state(&mut self.party[index].states, knockout, 0);
            }
        }
        self.mirror_party_to_battle();
    }

    pub(crate) fn change_mp(&mut self, target: ActorTarget, op: AmountOp, amount: i32) {
        let ids = self.target_ids(target);
        for index in ids {
            let max_mp = crate::party::stats_of(&self.game, &self.party[index]).mmp;
            let mp = &mut self.party[index].mp;
            *mp = match op {
                AmountOp::Gain => mp.saturating_add(amount.abs()).min(max_mp),
                AmountOp::Lose => mp.saturating_sub(amount.abs()).max(0),
            };
        }
        self.mirror_party_to_battle();
    }

    pub(crate) fn recover(&mut self, target: ActorTarget) {
        let ids = self.target_ids(target);
        for index in ids {
            let stats = crate::party::stats_of(&self.game, &self.party[index]);
            self.party[index].hp = stats.mhp;
            self.party[index].mp = stats.mmp;
            self.party[index].states.clear();
        }
        self.mirror_party_to_battle();
    }

    fn change_items(&mut self, id: u32, op: AmountOp, amount: i32) {
        let amount = amount.abs() as u32;
        match op {
            AmountOp::Gain => self.bag.gain_item(id, amount),
            AmountOp::Lose => {
                let _ = self.bag.lose_item(id, amount.min(self.bag.count_item(id)));
            }
        }
    }

    pub(crate) fn drive_foreground(&mut self) {
        let Some(mut process) = self.foreground.take() else {
            return;
        };
        self.run_interp(&mut process.interp);
        if process.interp.finished() {
            if let Some(id) = process.event_id {
                self.unlock_event(id);
            }
        } else {
            self.foreground = Some(process);
        }
    }

    pub(crate) fn drive_parallels(&mut self) {
        let mut parallels = std::mem::take(&mut self.parallels);
        let mut keep = Vec::new();
        for mut process in parallels.drain(..) {
            self.run_interp(&mut process.interp);
            if process.interp.finished() {
                if let Some(id) = process.event_id {
                    self.unlock_event(id);
                }
            } else {
                keep.push(process);
            }
        }
        self.parallels = keep;
    }

    pub(crate) fn clear_wait(&mut self, kind: WaitKind) {
        let clear = |wait: &mut Wait| {
            let matches = match kind {
                WaitKind::Message => matches!(wait, Wait::Message),
                WaitKind::Menu => matches!(wait, Wait::Menu),
                WaitKind::Save => matches!(wait, Wait::Save),
            };
            if matches {
                *wait = Wait::None;
            }
        };
        if let Some(process) = &mut self.foreground {
            clear(&mut process.interp.wait);
        }
        for process in &mut self.parallels {
            clear(&mut process.interp.wait);
        }
        if let Some(battle) = &mut self.battle {
            if let Some(interp) = &mut battle.event {
                clear(&mut interp.wait);
            }
        }
    }

    pub(crate) fn push_choice_branch(&mut self, commands: Vec<Command>) {
        let push = |interp: &mut Interp, commands: &[Command]| -> bool {
            if !matches!(interp.wait, Wait::Choice) {
                return false;
            }
            interp.wait = Wait::None;
            if !commands.is_empty() {
                interp.stack.push(Frame {
                    commands: commands.to_vec(),
                    ip: 0,
                });
            }
            true
        };
        let commands = commands;
        if let Some(process) = &mut self.foreground {
            if push(&mut process.interp, &commands) {
                return;
            }
        }
        for process in &mut self.parallels {
            if push(&mut process.interp, &commands) {
                return;
            }
        }
        if let Some(battle) = &mut self.battle {
            if let Some(interp) = &mut battle.event {
                let _ = push(interp, &commands);
            }
        }
    }

    pub(crate) fn release_battle_branch(&mut self, won: bool) {
        let Some(process) = &mut self.foreground else {
            return;
        };
        let wait = std::mem::replace(&mut process.interp.wait, Wait::None);
        if let Wait::Battle { on_win, on_escape } = wait {
            let branch = if won { on_win } else { on_escape };
            if !branch.is_empty() {
                process.interp.stack.push(Frame {
                    commands: branch,
                    ip: 0,
                });
            }
        } else {
            process.interp.wait = wait;
        }
    }

    fn route_running(&self, target: MoveTarget, interp: &Interp) -> bool {
        match self.resolve_target(target, interp) {
            MoveTarget::Player => self
                .player
                .route
                .as_ref()
                .is_some_and(|route| route.command),
            MoveTarget::ThisEvent => interp
                .event_id
                .is_some_and(|id| self.event_route_command(id)),
            MoveTarget::Event { id } => self.event_route_command(id),
        }
    }

    fn resolve_target(&self, target: MoveTarget, interp: &Interp) -> MoveTarget {
        match target {
            MoveTarget::ThisEvent => interp
                .event_id
                .map(|id| MoveTarget::Event { id })
                .unwrap_or(MoveTarget::ThisEvent),
            other => other,
        }
    }
}

#[derive(Clone, Copy)]
pub enum WaitKind {
    Message,
    Menu,
    Save,
}

pub fn add_state(states: &mut Vec<StateInst>, id: u32, turns: u32) {
    if id == 0 {
        return;
    }
    if let Some(existing) = states.iter_mut().find(|state| state.id == id) {
        existing.turns = turns;
    } else {
        states.push(StateInst { id, turns });
    }
}

pub fn remove_state(states: &mut Vec<StateInst>, id: u32) {
    states.retain(|state| state.id != id);
}

impl Process {
    pub fn event(map_id: u32, event_id: u32, commands: Vec<Command>) -> Self {
        let mut interp = Interp::new(commands);
        interp.map_id = map_id;
        interp.event_id = Some(event_id);
        Self {
            map_id,
            event_id: Some(event_id),
            common_id: None,
            page: None,
            interp,
        }
    }

    pub fn common(map_id: u32, common_id: u32, commands: Vec<Command>) -> Self {
        let mut interp = Interp::new(commands);
        interp.map_id = map_id;
        Self {
            map_id,
            event_id: None,
            common_id: Some(common_id),
            page: None,
            interp,
        }
    }
}

pub const TILE_PX: i32 = TILE as i32;
