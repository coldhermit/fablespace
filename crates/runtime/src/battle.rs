//! Turn-based front-view battle.

use fablespace_data::{
    ActionCondition, DamageKind, Effect, EnemyAction, Scope, Span, TroopCondition,
};

use crate::interpret::{add_state, remove_state};
use crate::party::{self, basic_attack, scaled_amount, StateInst, Stats};
use crate::session::{Input, Interp, Session};

#[derive(Clone, Debug)]
pub struct Fighter {
    pub name: String,
    pub hp: i32,
    pub mp: i32,
    pub stats: Stats,
    pub states: Vec<StateInst>,
    pub guarding: bool,
    pub skills: Vec<u32>,
    pub actor_index: Option<usize>,
    pub enemy_id: u32,
    pub x: i32,
    pub y: i32,
    pub flash: u8,
    pub actions: Vec<EnemyAction>,
    pub exp: i32,
    pub gold: i32,
    pub drops: Vec<(u32, u32)>,
}

#[derive(Clone, Debug)]
pub enum Phase {
    Boot,
    Command { cursor: usize },
    Skill { cursor: usize },
    Item { cursor: usize },
    Target { cursor: usize, pending: Pending },
    Resolving,
    Victory,
    Defeat,
    Escaped,
}

#[derive(Clone, Debug)]
pub enum Pending {
    Attack,
    Skill(u32),
    Item(u32),
}

#[derive(Clone, Debug)]
pub enum Aim {
    Enemy(usize),
    Actor(usize),
    AllEnemies,
    AllActors,
    User,
}

#[derive(Clone, Debug)]
pub enum Planned {
    Attack { actor: usize, enemy: usize },
    Guard { actor: usize },
    Skill { actor: usize, skill: u32, aim: Aim },
    Item { actor: usize, item: u32, aim: Aim },
    Enemy { index: usize, skill: u32, aim: Aim },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Outcome {
    Win,
    Escape,
    Lose,
}

pub struct Battle {
    pub troop_id: u32,
    pub can_escape: bool,
    pub actors: Vec<Fighter>,
    pub enemies: Vec<Fighter>,
    pub phase: Phase,
    pub turn: u32,
    pub actor_slot: usize,
    pub queue: Vec<Planned>,
    pub queue_index: usize,
    pub escape_tries: i32,
    pub ran_battle: Vec<usize>,
    pub ran_turn: Vec<(usize, u32)>,
    pub event: Option<Interp>,
    pub result: Option<Outcome>,
}

impl Session {
    pub(crate) fn start_battle(&mut self, troop_id: u32, can_escape: bool) {
        let Some(troop) = self.game.troop(troop_id).cloned() else {
            return;
        };
        let actors = self
            .party
            .iter()
            .enumerate()
            .map(|(index, member)| {
                let stats = party::stats_of(&self.game, member);
                let name = self
                    .game
                    .actor(member.actor_id)
                    .map(|actor| actor.name.clone())
                    .unwrap_or_else(|| format!("Actor {}", member.actor_id));
                Fighter {
                    name,
                    hp: member.hp,
                    mp: member.mp,
                    stats,
                    states: member.states.clone(),
                    guarding: false,
                    skills: party::known_skills(&self.game, member),
                    actor_index: Some(index),
                    enemy_id: 0,
                    x: 0,
                    y: 0,
                    flash: 0,
                    actions: Vec::new(),
                    exp: 0,
                    gold: 0,
                    drops: Vec::new(),
                }
            })
            .collect();
        let enemies = troop
            .members
            .iter()
            .filter_map(|member| {
                let enemy = self.game.enemy(member.enemy_id)?;
                let stats = Stats {
                    mhp: enemy.param(0).max(1),
                    mmp: enemy.param(1),
                    atk: enemy.param(2),
                    def: enemy.param(3),
                    mat: enemy.param(4),
                    mdf: enemy.param(5),
                    agi: enemy.param(6),
                    luk: enemy.param(7),
                };
                Some(Fighter {
                    name: enemy.name.clone(),
                    hp: stats.mhp,
                    mp: stats.mmp,
                    stats,
                    states: Vec::new(),
                    guarding: false,
                    skills: Vec::new(),
                    actor_index: None,
                    enemy_id: enemy.id,
                    x: member.x,
                    y: member.y,
                    flash: 0,
                    actions: enemy.actions.clone(),
                    exp: enemy.exp,
                    gold: enemy.gold,
                    drops: enemy
                        .drops
                        .iter()
                        .map(|drop| (drop.item_id, drop.denominator))
                        .collect(),
                })
            })
            .collect();
        self.battle = Some(Battle {
            troop_id,
            can_escape,
            actors,
            enemies,
            phase: Phase::Boot,
            turn: 0,
            actor_slot: 0,
            queue: Vec::new(),
            queue_index: 0,
            escape_tries: 0,
            ran_battle: Vec::new(),
            ran_turn: Vec::new(),
            event: None,
            result: None,
        });
        self.drive_battle_event();
    }

    pub(crate) fn update_battle(&mut self, input: &Input) {
        if self.battle.is_none() {
            return;
        }
        if self.battle.as_ref().unwrap().event.is_some() {
            self.drive_battle_event();
            if self.message.is_some() || self.choice.is_some() {
                return;
            }
        }
        if self.message.is_some() {
            return;
        }
        let phase = self.battle.as_ref().unwrap().phase.clone();
        match phase {
            Phase::Boot => self.boot_battle(),
            Phase::Command { cursor } => self.battle_command(input, cursor),
            Phase::Skill { cursor } => self.battle_skill(input, cursor),
            Phase::Item { cursor } => self.battle_item(input, cursor),
            Phase::Target { cursor, pending } => self.battle_target(input, cursor, pending),
            Phase::Resolving => self.resolve_one(),
            Phase::Victory | Phase::Defeat | Phase::Escaped => {}
        }
    }

    pub(crate) fn after_battle_message(&mut self) {
        if self.battle.is_none() {
            return;
        }
        if self
            .battle
            .as_ref()
            .is_some_and(|battle| battle.event.is_some())
        {
            self.drive_battle_event();
            if self.message.is_some() || self.choice.is_some() {
                return;
            }
        }
        let Some(phase) = self.battle.as_ref().map(|battle| battle.phase.clone()) else {
            return;
        };
        match phase {
            Phase::Boot => self.boot_battle(),
            Phase::Resolving => self.resolve_one(),
            Phase::Victory => self.finish_battle(Outcome::Win),
            Phase::Defeat => self.finish_battle(Outcome::Lose),
            Phase::Escaped => self.finish_battle(Outcome::Escape),
            _ => {}
        }
    }

    pub(crate) fn drive_battle_event(&mut self) {
        let waiting_boot = self
            .battle
            .as_ref()
            .is_some_and(|battle| battle.event.is_none() && matches!(battle.phase, Phase::Boot));
        if waiting_boot {
            let list = self.collect_troop_pages();
            if !list.is_empty() {
                if let Some(battle) = &mut self.battle {
                    battle.event = Some(Interp::new(list));
                }
            }
        }
        let Some(mut interp) = self.battle.as_mut().and_then(|battle| battle.event.take()) else {
            if waiting_boot && self.message.is_none() && self.battle.is_some() {
                self.begin_command();
            }
            return;
        };
        self.run_interp(&mut interp);
        let finished = interp.finished();
        if let Some(battle) = &mut self.battle {
            if !finished {
                battle.event = Some(interp);
            }
        }
        if finished
            && self.message.is_none()
            && self.choice.is_none()
            && self
                .battle
                .as_ref()
                .is_some_and(|battle| matches!(battle.phase, Phase::Boot))
        {
            self.begin_command();
        }
    }

    fn boot_battle(&mut self) {
        if self.living_enemies().is_empty() {
            self.begin_victory();
            return;
        }
        if self.living_actors().is_empty() {
            self.begin_defeat();
            return;
        }
        self.drive_battle_event();
    }

    fn begin_command(&mut self) {
        if self.living_enemies().is_empty() {
            self.begin_victory();
            return;
        }
        if self.living_actors().is_empty() {
            self.begin_defeat();
            return;
        }
        let Some(slot) = self.next_actor_from(0) else {
            self.queue_enemies_and_resolve();
            return;
        };
        if let Some(battle) = &mut self.battle {
            battle.actor_slot = slot;
            battle.phase = Phase::Command { cursor: 0 };
            battle.queue.clear();
            battle.queue_index = 0;
        }
    }

    fn battle_command(&mut self, input: &Input, mut cursor: usize) {
        let escape = self.battle.as_ref().is_some_and(|battle| battle.can_escape)
            && self.first_living_actor() == self.battle.as_ref().map(|battle| battle.actor_slot);
        let len = if escape { 5 } else { 4 };
        if input.dir == Some(fablespace_data::Direction::Up) {
            cursor = (cursor + len - 1) % len;
        }
        if input.dir == Some(fablespace_data::Direction::Down) {
            cursor = (cursor + 1) % len;
        }
        if let Some(battle) = &mut self.battle {
            battle.phase = Phase::Command { cursor };
        }
        if input.cancel && cursor != 0 {
            if let Some(battle) = &mut self.battle {
                battle.phase = Phase::Command { cursor: 0 };
            }
            return;
        }
        if !input.confirm {
            return;
        }
        match cursor {
            0 => self.pending_attack(),
            1 => self.plan_guard(),
            2 => {
                if let Some(battle) = &mut self.battle {
                    battle.phase = Phase::Skill { cursor: 0 };
                }
            }
            3 => {
                if let Some(battle) = &mut self.battle {
                    battle.phase = Phase::Item { cursor: 0 };
                }
            }
            _ => self.try_escape(),
        }
    }

    fn pending_attack(&mut self) {
        let enemies = self.living_enemies();
        if enemies.len() <= 1 {
            let enemy = enemies.first().copied().unwrap_or(0);
            self.push_party(Planned::Attack {
                actor: self.battle.as_ref().map(|b| b.actor_slot).unwrap_or(0),
                enemy,
            });
        } else if let Some(battle) = &mut self.battle {
            battle.phase = Phase::Target {
                cursor: 0,
                pending: Pending::Attack,
            };
        }
    }

    fn plan_guard(&mut self) {
        let actor = self.battle.as_ref().map(|b| b.actor_slot).unwrap_or(0);
        self.push_party(Planned::Guard { actor });
    }

    fn battle_skill(&mut self, input: &Input, mut cursor: usize) {
        let skills = self.command_skills();
        if skills.is_empty() {
            self.notify("No skills.");
            if let Some(battle) = &mut self.battle {
                battle.phase = Phase::Command { cursor: 2 };
            }
            return;
        }
        let len = skills.len();
        if input.dir == Some(fablespace_data::Direction::Up) {
            cursor = (cursor + len - 1) % len;
        }
        if input.dir == Some(fablespace_data::Direction::Down) {
            cursor = (cursor + 1) % len;
        }
        if let Some(battle) = &mut self.battle {
            battle.phase = Phase::Skill { cursor };
        }
        if input.cancel {
            if let Some(battle) = &mut self.battle {
                battle.phase = Phase::Command { cursor: 2 };
            }
            return;
        }
        if !input.confirm {
            return;
        }
        let Some(skill_id) = skills.get(cursor).copied() else {
            return;
        };
        let actor = self.battle.as_ref().map(|b| b.actor_slot).unwrap_or(0);
        let cost = self
            .game
            .skill(skill_id)
            .map(|skill| skill.mp_cost)
            .unwrap_or(0);
        let mp = self
            .battle
            .as_ref()
            .and_then(|b| b.actors.get(actor))
            .map(|f| f.mp)
            .unwrap_or(0);
        if mp < cost {
            self.notify("Not enough MP.");
            return;
        }
        self.aim_party_action(Pending::Skill(skill_id));
    }

    fn battle_item(&mut self, input: &Input, mut cursor: usize) {
        let items = self.battle_items();
        if items.is_empty() {
            self.notify("No items.");
            if let Some(battle) = &mut self.battle {
                battle.phase = Phase::Command { cursor: 3 };
            }
            return;
        }
        let len = items.len();
        if input.dir == Some(fablespace_data::Direction::Up) {
            cursor = (cursor + len - 1) % len;
        }
        if input.dir == Some(fablespace_data::Direction::Down) {
            cursor = (cursor + 1) % len;
        }
        if let Some(battle) = &mut self.battle {
            battle.phase = Phase::Item { cursor };
        }
        if input.cancel {
            if let Some(battle) = &mut self.battle {
                battle.phase = Phase::Command { cursor: 3 };
            }
            return;
        }
        if input.confirm {
            if let Some(id) = items.get(cursor).copied() {
                self.aim_party_action(Pending::Item(id));
            }
        }
    }

    fn aim_party_action(&mut self, pending: Pending) {
        let scope = match &pending {
            Pending::Attack => Scope::OneEnemy,
            Pending::Skill(id) => self
                .game
                .skill(*id)
                .map(|s| s.scope)
                .unwrap_or(Scope::OneEnemy),
            Pending::Item(id) => self.game.item(*id).map(|s| s.scope).unwrap_or(Scope::User),
        };
        let actor = self.battle.as_ref().map(|b| b.actor_slot).unwrap_or(0);
        match scope {
            Scope::OneEnemy => {
                let enemies = self.living_enemies();
                if enemies.len() <= 1 {
                    let aim = Aim::Enemy(enemies.first().copied().unwrap_or(0));
                    self.commit_pending(actor, pending, aim);
                } else if let Some(battle) = &mut self.battle {
                    battle.phase = Phase::Target { cursor: 0, pending };
                }
            }
            Scope::AllEnemies => self.commit_pending(actor, pending, Aim::AllEnemies),
            Scope::OneAlly => {
                let allies = self.living_actors();
                if allies.len() <= 1 {
                    self.commit_pending(
                        actor,
                        pending,
                        Aim::Actor(allies.first().copied().unwrap_or(actor)),
                    );
                } else if let Some(battle) = &mut self.battle {
                    battle.phase = Phase::Target { cursor: 0, pending };
                }
            }
            Scope::AllAllies => self.commit_pending(actor, pending, Aim::AllActors),
            Scope::User | Scope::None => self.commit_pending(actor, pending, Aim::User),
        }
    }

    fn battle_target(&mut self, input: &Input, mut cursor: usize, pending: Pending) {
        let allies = matches!(pending_scope(&self.game, &pending), Scope::OneAlly);
        let options = if allies {
            self.living_actors()
        } else {
            self.living_enemies()
        };
        if options.is_empty() {
            self.begin_command();
            return;
        }
        let len = options.len();
        if input.dir == Some(fablespace_data::Direction::Up) {
            cursor = (cursor + len - 1) % len;
        }
        if input.dir == Some(fablespace_data::Direction::Down) {
            cursor = (cursor + 1) % len;
        }
        if let Some(battle) = &mut self.battle {
            battle.phase = Phase::Target {
                cursor,
                pending: pending.clone(),
            };
        }
        if input.cancel {
            if let Some(battle) = &mut self.battle {
                battle.phase = Phase::Command { cursor: 0 };
            }
            return;
        }
        if input.confirm {
            let index = options[cursor];
            let actor = self.battle.as_ref().map(|b| b.actor_slot).unwrap_or(0);
            let aim = if allies {
                Aim::Actor(index)
            } else {
                Aim::Enemy(index)
            };
            self.commit_pending(actor, pending, aim);
        }
    }

    fn commit_pending(&mut self, actor: usize, pending: Pending, aim: Aim) {
        let planned = match pending {
            Pending::Attack => {
                let enemy = match aim {
                    Aim::Enemy(index) => index,
                    _ => 0,
                };
                Planned::Attack { actor, enemy }
            }
            Pending::Skill(skill) => Planned::Skill { actor, skill, aim },
            Pending::Item(item) => Planned::Item { actor, item, aim },
        };
        self.push_party(planned);
    }

    fn push_party(&mut self, planned: Planned) {
        let actor = match &planned {
            Planned::Attack { actor, .. }
            | Planned::Guard { actor }
            | Planned::Skill { actor, .. }
            | Planned::Item { actor, .. } => *actor,
            Planned::Enemy { .. } => 0,
        };
        if let Some(battle) = &mut self.battle {
            battle.queue.push(planned);
        }
        let next = self.next_actor_from(actor + 1);
        if let Some(slot) = next {
            if let Some(battle) = &mut self.battle {
                battle.actor_slot = slot;
                battle.phase = Phase::Command { cursor: 0 };
            }
        } else {
            self.queue_enemies_and_resolve();
        }
    }

    fn queue_enemies_and_resolve(&mut self) {
        let plans = self.enemy_plans();
        if let Some(battle) = &mut self.battle {
            battle.queue.extend(plans);
            let queued = std::mem::take(&mut battle.queue);
            let mut keyed: Vec<(i32, usize, Planned)> = queued
                .into_iter()
                .enumerate()
                .map(|(order, planned)| (plan_agi(battle, &planned), order, planned))
                .collect();
            keyed.sort_by(|a, b| b.0.cmp(&a.0).then(a.1.cmp(&b.1)));
            battle.queue = keyed.into_iter().map(|(_, _, planned)| planned).collect();
            battle.queue_index = 0;
            battle.phase = Phase::Resolving;
        }
        self.resolve_one();
    }

    fn enemy_plans(&mut self) -> Vec<Planned> {
        let Some(battle) = &self.battle else {
            return Vec::new();
        };
        let turn = battle.turn;
        let specs: Vec<(usize, Vec<fablespace_data::EnemyAction>, i32, i32)> = battle
            .enemies
            .iter()
            .enumerate()
            .filter(|(_, enemy)| alive(enemy))
            .map(|(index, enemy)| (index, enemy.actions.clone(), enemy.hp, enemy.stats.mhp))
            .collect();
        let mut plans = Vec::new();
        for (index, actions, hp, mhp) in specs {
            let skill = choose_action(&actions, hp, mhp, turn, &mut self.rng);
            let aim = if skill == 0 {
                Aim::Actor(self.random_living_actor())
            } else {
                let scope = self
                    .game
                    .skill(skill)
                    .map(|s| s.scope)
                    .unwrap_or(Scope::OneEnemy);
                match scope {
                    Scope::AllEnemies => Aim::AllActors,
                    Scope::AllAllies => Aim::AllEnemies,
                    Scope::OneAlly | Scope::User | Scope::None => Aim::Enemy(index),
                    Scope::OneEnemy => Aim::Actor(self.random_living_actor()),
                }
            };
            plans.push(Planned::Enemy { index, skill, aim });
        }
        plans
    }

    fn resolve_one(&mut self) {
        if self.victory_check() {
            return;
        }
        let done = self
            .battle
            .as_ref()
            .is_some_and(|battle| battle.queue_index >= battle.queue.len());
        if done {
            self.end_round();
            return;
        }
        let action = {
            let battle = self.battle.as_mut().unwrap();
            let action = battle.queue[battle.queue_index].clone();
            battle.queue_index += 1;
            action
        };
        let lines = self.apply_planned(action);
        if lines.is_empty() {
            self.resolve_one();
        } else {
            self.show_lines(String::new(), String::new(), lines);
        }
    }

    fn apply_planned(&mut self, action: Planned) -> Vec<String> {
        match action {
            Planned::Guard { actor } => {
                if let Some(fighter) = self.battle.as_mut().and_then(|b| b.actors.get_mut(actor)) {
                    if alive(fighter) {
                        fighter.guarding = true;
                        return vec![format!("{} guards.", fighter.name)];
                    }
                }
                Vec::new()
            }
            Planned::Attack { actor, enemy } => self.actor_attack(actor, enemy),
            Planned::Skill { actor, skill, aim } => self.actor_skill(actor, skill, aim),
            Planned::Item { actor, item, aim } => self.actor_item(actor, item, aim),
            Planned::Enemy { index, skill, aim } => self.enemy_act(index, skill, aim),
        }
    }

    fn actor_attack(&mut self, actor: usize, enemy: usize) -> Vec<String> {
        if !self.actor_can_act(actor) || !self.enemy_alive(enemy) {
            return Vec::new();
        }
        let user = self.battle.as_ref().unwrap().actors[actor].stats;
        let name = self.battle.as_ref().unwrap().actors[actor].name.clone();
        let target_name = self.battle.as_ref().unwrap().enemies[enemy].name.clone();
        if !self.rng.chance(100) {
            return vec![format!("{name} attacks!"), format!("{name} missed!")];
        }
        let target = self.battle.as_ref().unwrap().enemies[enemy].stats;
        let guard = self.battle.as_ref().unwrap().enemies[enemy].guarding;
        let roll = self.rng.percent();
        let amount = scaled_amount(&basic_attack(), user, target, roll);
        let dealt = self.hurt_enemy(enemy, amount, guard);
        let mut lines = vec![
            format!("{name} attacks!"),
            format!("{target_name} takes {dealt} damage!"),
        ];
        if self.battle.as_ref().unwrap().enemies[enemy].hp == 0 {
            lines.push(format!("{target_name} is defeated!"));
        }
        lines
    }

    fn actor_skill(&mut self, actor: usize, skill_id: u32, aim: Aim) -> Vec<String> {
        if !self.actor_can_act(actor) {
            return Vec::new();
        }
        let Some(skill) = self.game.skill(skill_id).cloned() else {
            return Vec::new();
        };
        let mp = self.battle.as_ref().unwrap().actors[actor].mp;
        if mp < skill.mp_cost {
            return vec!["Not enough MP.".into()];
        }
        if let Some(fighter) = self.battle.as_mut().unwrap().actors.get_mut(actor) {
            fighter.mp -= skill.mp_cost;
        }
        let name = self.battle.as_ref().unwrap().actors[actor].name.clone();
        let mut lines = vec![format!("{name} uses {}!", skill.name)];
        lines.extend(self.apply_usable(true, actor, aim, skill.hit, &skill.damage, &skill.effects));
        lines
    }

    fn actor_item(&mut self, actor: usize, item_id: u32, aim: Aim) -> Vec<String> {
        if !self.actor_can_act(actor) || self.bag.count_item(item_id) == 0 {
            return Vec::new();
        }
        let Some(item) = self.game.item(item_id).cloned() else {
            return Vec::new();
        };
        if item.consumable {
            let _ = self.bag.lose_item(item_id, 1);
        }
        let name = self.battle.as_ref().unwrap().actors[actor].name.clone();
        let mut lines = vec![format!("{name} uses {}!", item.name)];
        lines.extend(self.apply_usable(true, actor, aim, item.hit, &item.damage, &item.effects));
        lines
    }

    fn enemy_act(&mut self, index: usize, skill_id: u32, aim: Aim) -> Vec<String> {
        if !self.enemy_alive(index) {
            return Vec::new();
        }
        if skill_id == 0 {
            let target = match aim {
                Aim::Actor(i) => i,
                _ => self.random_living_actor(),
            };
            return self.enemy_attack(index, target);
        }
        let skill = self.game.skill(skill_id).cloned();
        let Some(skill) = skill else {
            let actor = self.random_living_actor();
            return self.enemy_attack(index, actor);
        };
        let name = self.battle.as_ref().unwrap().enemies[index].name.clone();
        let mut lines = vec![format!("{name} uses {}!", skill.name)];
        lines.extend(self.apply_usable(
            false,
            index,
            aim,
            skill.hit,
            &skill.damage,
            &skill.effects,
        ));
        lines
    }

    fn enemy_attack(&mut self, index: usize, actor: usize) -> Vec<String> {
        if !self.enemy_alive(index) || !self.actor_can_act(actor) {
            return Vec::new();
        }
        let name = self.battle.as_ref().unwrap().enemies[index].name.clone();
        let target_name = self.battle.as_ref().unwrap().actors[actor].name.clone();
        if !self.rng.chance(100) {
            return vec![format!("{name} attacks!"), format!("{name} missed!")];
        }
        let user = self.battle.as_ref().unwrap().enemies[index].stats;
        let target = self.battle.as_ref().unwrap().actors[actor].stats;
        let guard = self.battle.as_ref().unwrap().actors[actor].guarding;
        let amount = scaled_amount(&basic_attack(), user, target, self.rng.percent());
        let dealt = self.hurt_actor(actor, amount, guard);
        let mut lines = vec![
            format!("{name} attacks!"),
            format!("{target_name} takes {dealt} damage!"),
        ];
        if self.battle.as_ref().unwrap().actors[actor].hp == 0 {
            lines.push(format!("{target_name} is knocked out!"));
        }
        lines
    }

    fn apply_usable(
        &mut self,
        from_actor: bool,
        user_index: usize,
        aim: Aim,
        hit: i32,
        damage: &fablespace_data::Damage,
        effects: &[Effect],
    ) -> Vec<String> {
        let targets = self.expand_aim(from_actor, user_index, &aim);
        let user_stats = if from_actor {
            self.battle.as_ref().unwrap().actors[user_index].stats
        } else {
            self.battle.as_ref().unwrap().enemies[user_index].stats
        };
        let mut lines = Vec::new();
        if targets.is_empty() {
            return lines;
        }
        for target in targets {
            if !self.rng.chance(hit) {
                lines.push("Miss!".into());
                continue;
            }
            let (name, stats, guarding) = self.fighter_view(target);
            if damage.kind != DamageKind::None {
                let amount = scaled_amount(damage, user_stats, stats, self.rng.percent());
                match damage.kind {
                    DamageKind::HpDamage => {
                        let dealt = self.hurt(target, amount, guarding);
                        lines.push(format!("{name} takes {dealt} damage!"));
                    }
                    DamageKind::MpDamage => {
                        self.spend_mp(target, amount);
                        lines.push(format!("{name} loses {amount} MP!"));
                    }
                    DamageKind::HpRecover => {
                        let gained = self.heal_hp(target, amount);
                        lines.push(format!("{name} recovers {gained} HP!"));
                    }
                    DamageKind::MpRecover => {
                        let gained = self.heal_mp(target, amount);
                        lines.push(format!("{name} recovers {gained} MP!"));
                    }
                    DamageKind::None => {}
                }
            }
            for effect in effects {
                lines.extend(self.apply_effect(target, effect));
            }
            if self.fighter_hp(target) == 0 {
                lines.push(format!("{name} is knocked out!"));
            }
        }
        lines
    }

    fn apply_effect(&mut self, target: TargetRef, effect: &Effect) -> Vec<String> {
        let name = self.fighter_view(target).0;
        match effect {
            Effect::RecoverHp { percent, flat } => {
                let max = self.fighter_view(target).1.mhp;
                let amount = max * percent / 100 + flat;
                let gained = self.heal_hp(target, amount);
                if gained > 0 {
                    vec![format!("{name} recovers {gained} HP!")]
                } else {
                    Vec::new()
                }
            }
            Effect::RecoverMp { percent, flat } => {
                let max = self.fighter_view(target).1.mmp;
                let amount = max * percent / 100 + flat;
                let gained = self.heal_mp(target, amount);
                if gained > 0 {
                    vec![format!("{name} recovers {gained} MP!")]
                } else {
                    Vec::new()
                }
            }
            Effect::AddState { state_id, chance } => {
                if !self.rng.chance(*chance) {
                    return Vec::new();
                }
                let turns = self.state_turns(*state_id);
                self.add_fighter_state(target, *state_id, turns);
                self.game
                    .state(*state_id)
                    .map(|state| vec![format!("{name} is {}!", state.name.to_lowercase())])
                    .unwrap_or_default()
            }
            Effect::RemoveState { state_id } => {
                self.remove_fighter_state(target, *state_id);
                Vec::new()
            }
        }
    }

    fn end_round(&mut self) {
        let mut lines = Vec::new();
        let knockout = self.knockout_state();
        let slips = self.slip_amounts();
        for (target, amount) in slips {
            if amount <= 0 {
                continue;
            }
            let name = self.fighter_view(target).0;
            let dealt = self.hurt(target, amount, false);
            lines.push(format!("{name} is hurt by poison for {dealt}!"));
            if self.fighter_hp(target) == 0 {
                lines.push(format!("{name} is knocked out!"));
            }
        }
        self.tick_states(knockout);
        if let Some(battle) = &mut self.battle {
            for fighter in battle.actors.iter_mut().chain(battle.enemies.iter_mut()) {
                fighter.guarding = false;
            }
            battle.turn += 1;
            battle.phase = Phase::Boot;
            battle.queue.clear();
        }
        if self.victory_check() {
            return;
        }
        if lines.is_empty() {
            self.boot_battle();
        } else {
            self.show_lines(String::new(), String::new(), lines);
        }
    }

    fn victory_check(&mut self) -> bool {
        if self.living_enemies().is_empty() && self.battle.is_some() {
            self.begin_victory();
            true
        } else if self.living_actors().is_empty() && self.battle.is_some() {
            self.begin_defeat();
            true
        } else {
            false
        }
    }

    fn begin_victory(&mut self) {
        self.write_fighters_back();
        let mut lines = vec!["Victory!".into()];
        let (exp, gold, drops) = self.battle_rewards();
        self.gold = self.gold.saturating_add(gold);
        if gold > 0 {
            lines.push(format!("Found {gold} {}!", self.game.system.currency));
        }
        for (item_id, count) in drops {
            self.bag.gain_item(item_id, count);
            if let Some(item) = self.game.item(item_id) {
                lines.push(format!("Received {}!", item.name));
            }
        }
        let party_names: Vec<(usize, String)> = self
            .party
            .iter()
            .enumerate()
            .filter(|(_, member)| member.hp > 0)
            .map(|(index, member)| {
                let name = self
                    .game
                    .actor(member.actor_id)
                    .map(|actor| actor.name.clone())
                    .unwrap_or_default();
                (index, name)
            })
            .collect();
        for (index, name) in party_names {
            if exp > 0 {
                lines.push(format!("{name} gained {exp} EXP."));
            }
            lines.extend(party::grant_exp(&self.game, &mut self.party[index], exp));
        }
        if let Some(battle) = &mut self.battle {
            battle.phase = Phase::Victory;
        }
        self.show_lines(String::new(), String::new(), lines);
    }

    fn begin_defeat(&mut self) {
        self.write_fighters_back();
        if let Some(battle) = &mut self.battle {
            battle.phase = Phase::Defeat;
        }
        self.show_lines(
            String::new(),
            String::new(),
            vec!["The party was defeated.".into()],
        );
    }

    fn try_escape(&mut self) {
        let chance = self.escape_chance();
        if let Some(battle) = &mut self.battle {
            battle.escape_tries += 1;
        }
        if self.rng.chance(chance) {
            self.write_fighters_back();
            if let Some(battle) = &mut self.battle {
                battle.phase = Phase::Escaped;
            }
            self.show_lines(
                String::new(),
                String::new(),
                vec!["Got away safely.".into()],
            );
        } else {
            self.show_lines(
                String::new(),
                String::new(),
                vec!["Couldn't escape!".into()],
            );
            let plans = self.enemy_plans();
            if let Some(battle) = &mut self.battle {
                battle.queue = plans;
                battle.queue_index = 0;
                battle.phase = Phase::Resolving;
            }
        }
    }

    fn escape_chance(&self) -> i32 {
        let Some(battle) = &self.battle else {
            return 50;
        };
        let party: i32 = battle
            .actors
            .iter()
            .filter(|f| alive(f))
            .map(|f| f.stats.agi)
            .sum();
        let enemy: i32 = battle
            .enemies
            .iter()
            .filter(|f| alive(f))
            .map(|f| f.stats.agi)
            .sum();
        let base = 50 + battle.escape_tries * 10 + (party - enemy) / 2;
        base.clamp(10, 95)
    }

    fn finish_battle(&mut self, outcome: Outcome) {
        self.battle = None;
        match outcome {
            Outcome::Lose => self.enter_game_over(),
            Outcome::Win => {
                self.release_battle_branch(true);
                self.drive_foreground();
            }
            Outcome::Escape => {
                self.release_battle_branch(false);
                self.drive_foreground();
            }
        }
    }

    pub(crate) fn mirror_party_to_battle(&mut self) {
        let snapshot: Vec<(usize, i32, i32, Vec<StateInst>)> = self
            .party
            .iter()
            .enumerate()
            .map(|(index, member)| (index, member.hp, member.mp, member.states.clone()))
            .collect();
        let Some(battle) = &mut self.battle else {
            return;
        };
        for fighter in &mut battle.actors {
            if let Some(index) = fighter.actor_index {
                if let Some((_, hp, mp, states)) = snapshot.iter().find(|(i, _, _, _)| *i == index)
                {
                    fighter.hp = *hp;
                    fighter.mp = *mp;
                    fighter.states = states.clone();
                    fighter.stats = party::stats_of(&self.game, &self.party[index]);
                }
            }
        }
    }

    fn write_fighters_back(&mut self) {
        let snapshot = self.battle.as_ref().map(|battle| {
            battle
                .actors
                .iter()
                .filter_map(|fighter| {
                    fighter
                        .actor_index
                        .map(|index| (index, fighter.hp, fighter.mp, fighter.states.clone()))
                })
                .collect::<Vec<_>>()
        });
        let Some(snapshot) = snapshot else {
            return;
        };
        for (index, hp, mp, states) in snapshot {
            if let Some(member) = self.party.get_mut(index) {
                member.hp = hp;
                member.mp = mp;
                member.states = states;
            }
        }
    }

    fn battle_rewards(&mut self) -> (i32, i32, Vec<(u32, u32)>) {
        let Some(battle) = &self.battle else {
            return (0, 0, Vec::new());
        };
        let mut exp = 0;
        let mut gold = 0;
        let mut drops = Vec::new();
        for enemy in &battle.enemies {
            exp += enemy.exp.max(0);
            gold += enemy.gold.max(0);
            for (item_id, denominator) in &enemy.drops {
                if *denominator > 0 && self.rng.below(*denominator) == 0 {
                    drops.push((*item_id, 1));
                }
            }
        }
        (exp, gold, drops)
    }

    pub(crate) fn tick_flashes(&mut self) {
        let Some(battle) = &mut self.battle else {
            return;
        };
        for fighter in battle.actors.iter_mut().chain(battle.enemies.iter_mut()) {
            if fighter.flash > 0 {
                fighter.flash -= 1;
            }
        }
    }

    fn collect_troop_pages(&mut self) -> Vec<fablespace_data::Command> {
        let Some(battle) = &self.battle else {
            return Vec::new();
        };
        let troop = match self.game.troop(battle.troop_id).cloned() {
            Some(troop) => troop,
            None => return Vec::new(),
        };
        let turn = battle.turn;
        let mut list = Vec::new();
        let mut newly_battle = Vec::new();
        let mut newly_turn = Vec::new();
        for (index, page) in troop.pages.iter().enumerate() {
            if !troop_page_ok(&page.condition, turn, &battle.enemies, &self.switches) {
                continue;
            }
            match page.span {
                Span::Battle => {
                    if battle.ran_battle.contains(&index) {
                        continue;
                    }
                    newly_battle.push(index);
                    list.extend(page.list.clone());
                }
                Span::Turn => {
                    if battle
                        .ran_turn
                        .iter()
                        .any(|(i, t)| *i == index && *t == turn)
                    {
                        continue;
                    }
                    newly_turn.push((index, turn));
                    list.extend(page.list.clone());
                }
            }
        }
        if let Some(battle) = &mut self.battle {
            battle.ran_battle.extend(newly_battle);
            battle.ran_turn.extend(newly_turn);
        }
        list
    }

    pub(crate) fn command_skills(&self) -> Vec<u32> {
        let Some(battle) = &self.battle else {
            return Vec::new();
        };
        let Some(actor) = battle.actors.get(battle.actor_slot) else {
            return Vec::new();
        };
        actor
            .skills
            .iter()
            .copied()
            .filter(|id| {
                self.game
                    .skill(*id)
                    .is_some_and(|skill| skill.occasion.allows_battle())
            })
            .collect()
    }

    pub(crate) fn battle_items(&self) -> Vec<u32> {
        self.bag
            .items
            .iter()
            .filter(|(id, count)| {
                **count > 0
                    && self
                        .game
                        .item(**id)
                        .is_some_and(|item| item.occasion.allows_battle())
            })
            .map(|(id, _)| *id)
            .collect()
    }

    fn living_actors(&self) -> Vec<usize> {
        self.battle
            .as_ref()
            .map(|battle| {
                battle
                    .actors
                    .iter()
                    .enumerate()
                    .filter(|(_, fighter)| alive(fighter) && !cannot_act(fighter, &self.game))
                    .map(|(index, _)| index)
                    .collect()
            })
            .unwrap_or_default()
    }

    fn living_enemies(&self) -> Vec<usize> {
        self.battle
            .as_ref()
            .map(|battle| {
                battle
                    .enemies
                    .iter()
                    .enumerate()
                    .filter(|(_, fighter)| alive(fighter))
                    .map(|(index, _)| index)
                    .collect()
            })
            .unwrap_or_default()
    }

    fn first_living_actor(&self) -> Option<usize> {
        self.living_actors().into_iter().next()
    }

    fn next_actor_from(&self, start: usize) -> Option<usize> {
        self.living_actors()
            .into_iter()
            .find(|index| *index >= start)
    }

    fn random_living_actor(&mut self) -> usize {
        let actors = self.living_actors();
        if actors.is_empty() {
            return 0;
        }
        let pick = self.rng.below(actors.len() as u32) as usize;
        actors[pick]
    }

    fn actor_can_act(&self, index: usize) -> bool {
        self.battle
            .as_ref()
            .and_then(|battle| battle.actors.get(index))
            .is_some_and(|fighter| alive(fighter) && !cannot_act(fighter, &self.game))
    }

    fn enemy_alive(&self, index: usize) -> bool {
        self.battle
            .as_ref()
            .and_then(|battle| battle.enemies.get(index))
            .is_some_and(alive)
    }

    fn expand_aim(&self, from_actor: bool, user_index: usize, aim: &Aim) -> Vec<TargetRef> {
        match aim {
            Aim::Enemy(index) => vec![TargetRef::Enemy(*index)],
            Aim::Actor(index) => vec![TargetRef::Actor(*index)],
            Aim::AllEnemies => self
                .living_enemies()
                .into_iter()
                .map(TargetRef::Enemy)
                .collect(),
            Aim::AllActors => (0..self.battle.as_ref().map(|b| b.actors.len()).unwrap_or(0))
                .map(TargetRef::Actor)
                .collect(),
            Aim::User => vec![if from_actor {
                TargetRef::Actor(user_index)
            } else {
                TargetRef::Enemy(user_index)
            }],
        }
    }

    fn fighter_view(&self, target: TargetRef) -> (String, Stats, bool) {
        let fighter = self.fighter(target);
        (
            fighter.map(|f| f.name.clone()).unwrap_or_default(),
            fighter.map(|f| f.stats).unwrap_or_default(),
            fighter.map(|f| f.guarding).unwrap_or(false),
        )
    }

    fn fighter(&self, target: TargetRef) -> Option<&Fighter> {
        let battle = self.battle.as_ref()?;
        match target {
            TargetRef::Actor(index) => battle.actors.get(index),
            TargetRef::Enemy(index) => battle.enemies.get(index),
        }
    }

    fn fighter_mut(&mut self, target: TargetRef) -> Option<&mut Fighter> {
        let battle = self.battle.as_mut()?;
        match target {
            TargetRef::Actor(index) => battle.actors.get_mut(index),
            TargetRef::Enemy(index) => battle.enemies.get_mut(index),
        }
    }

    fn fighter_hp(&self, target: TargetRef) -> i32 {
        self.fighter(target).map(|f| f.hp).unwrap_or(0)
    }

    fn hurt_actor(&mut self, index: usize, amount: i32, guard: bool) -> i32 {
        self.hurt(TargetRef::Actor(index), amount, guard)
    }

    fn hurt_enemy(&mut self, index: usize, amount: i32, guard: bool) -> i32 {
        self.hurt(TargetRef::Enemy(index), amount, guard)
    }

    fn hurt(&mut self, target: TargetRef, amount: i32, guard: bool) -> i32 {
        let knockout = self.knockout_state();
        let on_damage: Vec<u32> = self
            .game
            .states
            .iter()
            .filter(|state| matches!(state.removal, fablespace_data::Removal::OnDamage))
            .map(|state| state.id)
            .collect();
        let Some(fighter) = self.fighter_mut(target) else {
            return 0;
        };
        let dealt = if guard && fighter.guarding {
            amount / 2
        } else {
            amount
        }
        .max(0);
        fighter.hp = fighter.hp.saturating_sub(dealt);
        fighter.flash = 10;
        if fighter.hp == 0 && knockout != 0 {
            add_state(&mut fighter.states, knockout, 0);
        }
        let strip: Vec<u32> = fighter
            .states
            .iter()
            .map(|state| state.id)
            .filter(|id| on_damage.contains(id))
            .collect();
        for id in strip {
            remove_state(&mut fighter.states, id);
        }
        dealt
    }

    fn heal_hp(&mut self, target: TargetRef, amount: i32) -> i32 {
        let knockout = self.knockout_state();
        let Some(fighter) = self.fighter_mut(target) else {
            return 0;
        };
        if fighter.states.iter().any(|state| state.id == knockout) {
            return 0;
        }
        let before = fighter.hp;
        fighter.hp = (fighter.hp + amount.max(0)).min(fighter.stats.mhp);
        fighter.hp - before
    }

    fn heal_mp(&mut self, target: TargetRef, amount: i32) -> i32 {
        let Some(fighter) = self.fighter_mut(target) else {
            return 0;
        };
        let before = fighter.mp;
        fighter.mp = (fighter.mp + amount.max(0)).min(fighter.stats.mmp);
        fighter.mp - before
    }

    fn spend_mp(&mut self, target: TargetRef, amount: i32) {
        if let Some(fighter) = self.fighter_mut(target) {
            fighter.mp = fighter.mp.saturating_sub(amount.max(0));
        }
    }

    fn add_fighter_state(&mut self, target: TargetRef, id: u32, turns: u32) {
        let lethal = self.knockout_state() == id
            && self
                .game
                .state(id)
                .is_some_and(|state| state.restriction == fablespace_data::Restriction::CannotAct);
        if let Some(fighter) = self.fighter_mut(target) {
            add_state(&mut fighter.states, id, turns);
            if lethal && fighter.hp > 0 {
                fighter.hp = 0;
            }
        }
    }

    fn remove_fighter_state(&mut self, target: TargetRef, id: u32) {
        if let Some(fighter) = self.fighter_mut(target) {
            remove_state(&mut fighter.states, id);
        }
    }

    fn state_turns(&mut self, id: u32) -> u32 {
        match self.game.state(id).map(|state| state.removal.clone()) {
            Some(fablespace_data::Removal::Turns { min, max }) => {
                if max <= min {
                    min
                } else {
                    self.rng.range(min as i32, max as i32) as u32
                }
            }
            _ => 0,
        }
    }

    fn slip_amounts(&self) -> Vec<(TargetRef, i32)> {
        let Some(battle) = &self.battle else {
            return Vec::new();
        };
        let mut out = Vec::new();
        for (index, fighter) in battle.actors.iter().enumerate() {
            let amount = slip_of(fighter, &self.game);
            if amount > 0 && alive(fighter) {
                out.push((TargetRef::Actor(index), amount));
            }
        }
        for (index, fighter) in battle.enemies.iter().enumerate() {
            let amount = slip_of(fighter, &self.game);
            if amount > 0 && alive(fighter) {
                out.push((TargetRef::Enemy(index), amount));
            }
        }
        out
    }

    fn tick_states(&mut self, knockout: u32) {
        let timed: Vec<u32> = self
            .game
            .states
            .iter()
            .filter(|state| matches!(state.removal, fablespace_data::Removal::Turns { .. }))
            .map(|state| state.id)
            .collect();
        let Some(battle) = &mut self.battle else {
            return;
        };
        for fighter in battle.actors.iter_mut().chain(battle.enemies.iter_mut()) {
            let mut keep = Vec::new();
            for mut state in fighter.states.drain(..) {
                if state.id == knockout {
                    keep.push(state);
                    continue;
                }
                if timed.contains(&state.id) {
                    if state.turns > 0 {
                        state.turns -= 1;
                    }
                    if state.turns > 0 {
                        keep.push(state);
                    }
                } else {
                    keep.push(state);
                }
            }
            fighter.states = keep;
        }
    }
}

#[derive(Clone, Copy)]
enum TargetRef {
    Actor(usize),
    Enemy(usize),
}

fn alive(fighter: &Fighter) -> bool {
    fighter.hp > 0
}

fn cannot_act(fighter: &Fighter, game: &fablespace_data::Game) -> bool {
    fighter.states.iter().any(|state| {
        game.state(state.id)
            .is_some_and(|def| def.restriction == fablespace_data::Restriction::CannotAct)
    })
}

fn slip_of(fighter: &Fighter, game: &fablespace_data::Game) -> i32 {
    fighter.states.iter().fold(0, |total, state| {
        let percent = game
            .state(state.id)
            .map(|def| def.slip_hp_percent)
            .unwrap_or(0);
        if percent > 0 {
            total + (fighter.stats.mhp * percent / 100).max(1)
        } else {
            total
        }
    })
}

fn choose_action(
    actions: &[fablespace_data::EnemyAction],
    hp: i32,
    mhp: i32,
    turn: u32,
    rng: &mut crate::rng::Rng,
) -> u32 {
    let mut bag = Vec::new();
    for action in actions {
        if action_ok(action, hp, mhp, turn) {
            for _ in 0..action.rating.max(1) {
                bag.push(action.skill_id);
            }
        }
    }
    if bag.is_empty() {
        0
    } else {
        bag[rng.below(bag.len() as u32) as usize]
    }
}

fn action_ok(action: &EnemyAction, hp: i32, mhp: i32, turn: u32) -> bool {
    match &action.condition {
        ActionCondition::Always => true,
        ActionCondition::HpBelow { percent } => hp * 100 <= mhp * percent,
        ActionCondition::Turn { start, every } => {
            turn >= *start && (*every <= 1 || (turn - start) % every.max(&1) == 0)
        }
    }
}

fn plan_agi(battle: &Battle, planned: &Planned) -> i32 {
    match planned {
        Planned::Attack { actor, .. }
        | Planned::Guard { actor }
        | Planned::Skill { actor, .. }
        | Planned::Item { actor, .. } => {
            battle.actors.get(*actor).map(|f| f.stats.agi).unwrap_or(0)
        }
        Planned::Enemy { index, .. } => {
            battle.enemies.get(*index).map(|f| f.stats.agi).unwrap_or(0)
        }
    }
}

fn troop_page_ok(cond: &TroopCondition, turn: u32, enemies: &[Fighter], switches: &[bool]) -> bool {
    if cond.switch_id != 0
        && !switches
            .get(cond.switch_id as usize)
            .copied()
            .unwrap_or(false)
    {
        return false;
    }
    if let Some((index, percent)) = cond.enemy_hp {
        let Some(enemy) = enemies.get(index) else {
            return false;
        };
        if enemy.hp * 100 > enemy.stats.mhp * percent {
            return false;
        }
    }
    if let Some(start) = cond.turn_start {
        if turn < start {
            return false;
        }
        if cond.turn_every == 0 {
            if turn != start {
                return false;
            }
        } else if (turn - start) % cond.turn_every != 0 {
            return false;
        }
    }
    true
}

fn pending_scope(game: &fablespace_data::Game, pending: &Pending) -> Scope {
    match pending {
        Pending::Attack => Scope::OneEnemy,
        Pending::Skill(id) => game
            .skill(*id)
            .map(|skill| skill.scope)
            .unwrap_or(Scope::OneEnemy),
        Pending::Item(id) => game.item(*id).map(|item| item.scope).unwrap_or(Scope::User),
    }
}

/// Field use of an item or skill. Shared with the menu.
pub fn field_recover(
    session: &mut Session,
    actor_index: usize,
    effects: &[Effect],
    damage: &fablespace_data::Damage,
) -> Vec<String> {
    let mut lines = Vec::new();
    let member_stats = party::stats_of(&session.game, &session.party[actor_index]);
    let name = session
        .game
        .actor(session.party[actor_index].actor_id)
        .map(|actor| actor.name.clone())
        .unwrap_or_default();
    if matches!(damage.kind, DamageKind::HpRecover) {
        let amount = scaled_amount(damage, member_stats, member_stats, 100);
        let knockout = session.knockout_state();
        if !session.party[actor_index]
            .states
            .iter()
            .any(|state| state.id == knockout)
        {
            let before = session.party[actor_index].hp;
            session.party[actor_index].hp = (before + amount).min(member_stats.mhp);
            lines.push(format!(
                "{name} recovers {} HP.",
                session.party[actor_index].hp - before
            ));
        }
    }
    if matches!(damage.kind, DamageKind::MpRecover) {
        let amount = scaled_amount(damage, member_stats, member_stats, 100);
        let before = session.party[actor_index].mp;
        session.party[actor_index].mp = (before + amount).min(member_stats.mmp);
        lines.push(format!(
            "{name} recovers {} MP.",
            session.party[actor_index].mp - before
        ));
    }
    for effect in effects {
        match effect {
            Effect::RecoverHp { percent, flat } => {
                let knockout = session.knockout_state();
                if session.party[actor_index]
                    .states
                    .iter()
                    .any(|state| state.id == knockout)
                {
                    continue;
                }
                let amount = member_stats.mhp * percent / 100 + flat;
                let before = session.party[actor_index].hp;
                session.party[actor_index].hp = (before + amount.max(0)).min(member_stats.mhp);
                let gained = session.party[actor_index].hp - before;
                if gained > 0 {
                    lines.push(format!("{name} recovers {gained} HP."));
                }
            }
            Effect::RecoverMp { percent, flat } => {
                let amount = member_stats.mmp * percent / 100 + flat;
                let before = session.party[actor_index].mp;
                session.party[actor_index].mp = (before + amount.max(0)).min(member_stats.mmp);
                let gained = session.party[actor_index].mp - before;
                if gained > 0 {
                    lines.push(format!("{name} recovers {gained} MP."));
                }
            }
            Effect::RemoveState { state_id } => {
                remove_state(&mut session.party[actor_index].states, *state_id)
            }
            Effect::AddState { .. } => {}
        }
    }
    if lines.is_empty() {
        lines.push("Nothing happens.".into());
    }
    lines
}
