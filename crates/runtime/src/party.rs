//! Party members, equipment stats, and the fixed damage formula.

use fablespace_data::{exp_to_next, Damage, DamageKind, Element, Game, MHP};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StateInst {
    pub id: u32,
    pub turns: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Member {
    pub actor_id: u32,
    pub level: u32,
    pub exp: i32,
    pub hp: i32,
    pub mp: i32,
    pub weapon_id: u32,
    pub shield_id: u32,
    pub head_id: u32,
    pub body_id: u32,
    pub accessory_id: u32,
    pub states: Vec<StateInst>,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Stats {
    pub mhp: i32,
    pub mmp: i32,
    pub atk: i32,
    pub def: i32,
    pub mat: i32,
    pub mdf: i32,
    pub agi: i32,
    pub luk: i32,
}

impl Stats {
    fn from_values(v: [i32; 8]) -> Self {
        Self {
            mhp: v[0],
            mmp: v[1],
            atk: v[2],
            def: v[3],
            mat: v[4],
            mdf: v[5],
            agi: v[6],
            luk: v[7],
        }
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Bag {
    pub items: BTreeMap<u32, u32>,
    pub weapons: BTreeMap<u32, u32>,
    pub armors: BTreeMap<u32, u32>,
}

impl Bag {
    pub fn count_item(&self, id: u32) -> u32 {
        self.items.get(&id).copied().unwrap_or(0)
    }

    pub fn gain_item(&mut self, id: u32, amount: u32) {
        if id == 0 || amount == 0 {
            return;
        }
        *self.items.entry(id).or_insert(0) += amount;
    }

    pub fn lose_item(&mut self, id: u32, amount: u32) -> bool {
        let have = self.count_item(id);
        if have < amount {
            return false;
        }
        if have == amount {
            self.items.remove(&id);
        } else {
            self.items.insert(id, have - amount);
        }
        true
    }

    pub fn gain_weapon(&mut self, id: u32, amount: u32) {
        if id == 0 || amount == 0 {
            return;
        }
        *self.weapons.entry(id).or_insert(0) += amount;
    }

    pub fn lose_weapon(&mut self, id: u32, amount: u32) -> bool {
        let have = self.weapons.get(&id).copied().unwrap_or(0);
        if have < amount {
            return false;
        }
        if have == amount {
            self.weapons.remove(&id);
        } else {
            self.weapons.insert(id, have - amount);
        }
        true
    }

    pub fn gain_armor(&mut self, id: u32, amount: u32) {
        if id == 0 || amount == 0 {
            return;
        }
        *self.armors.entry(id).or_insert(0) += amount;
    }

    pub fn lose_armor(&mut self, id: u32, amount: u32) -> bool {
        let have = self.armors.get(&id).copied().unwrap_or(0);
        if have < amount {
            return false;
        }
        if have == amount {
            self.armors.remove(&id);
        } else {
            self.armors.insert(id, have - amount);
        }
        true
    }
}

pub fn stats_of(game: &Game, member: &Member) -> Stats {
    let class = game
        .actor(member.actor_id)
        .and_then(|actor| game.class(actor.class_id));
    let mut values = [0; 8];
    for index in 0..8 {
        let mut value = class
            .map(|class| class.growth(index).at(member.level))
            .unwrap_or(1);
        if let Some(weapon) = game.weapon(member.weapon_id) {
            value += param_at(&weapon.params, index);
        }
        for id in [
            member.shield_id,
            member.head_id,
            member.body_id,
            member.accessory_id,
        ] {
            if let Some(armor) = game.armor(id) {
                value += param_at(&armor.params, index);
            }
        }
        values[index] = if index == MHP {
            value.max(1)
        } else {
            value.max(0)
        };
    }
    Stats::from_values(values)
}

fn param_at(params: &[i32], index: usize) -> i32 {
    params.get(index).copied().unwrap_or(0)
}

pub fn member_from_actor(game: &Game, actor_id: u32) -> Option<Member> {
    let actor = game.actor(actor_id)?;
    let max_level = actor.max_level.max(1);
    let mut member = Member {
        actor_id,
        level: actor.initial_level.clamp(1, max_level),
        exp: 0,
        hp: 1,
        mp: 0,
        weapon_id: actor.weapon_id,
        shield_id: actor.shield_id,
        head_id: actor.head_id,
        body_id: actor.body_id,
        accessory_id: actor.accessory_id,
        states: Vec::new(),
    };
    let stats = stats_of(game, &member);
    member.hp = stats.mhp;
    member.mp = stats.mmp;
    Some(member)
}

pub fn known_skills(game: &Game, member: &Member) -> Vec<u32> {
    let Some(actor) = game.actor(member.actor_id) else {
        return Vec::new();
    };
    let Some(class) = game.class(actor.class_id) else {
        return Vec::new();
    };
    class
        .learnings
        .iter()
        .filter(|learning| learning.level <= member.level && learning.skill_id != 0)
        .map(|learning| learning.skill_id)
        .collect()
}

pub fn grant_exp(game: &Game, member: &mut Member, amount: i32) -> Vec<String> {
    let mut notes = Vec::new();
    if amount > 0 {
        member.exp = member.exp.saturating_add(amount);
    }
    let Some(actor) = game.actor(member.actor_id).cloned() else {
        return notes;
    };
    let Some(class) = game.class(actor.class_id).cloned() else {
        return notes;
    };
    while member.level < actor.max_level {
        let need = exp_to_next(&class.exp, member.level);
        if member.exp < need {
            break;
        }
        member.exp -= need;
        let before = stats_of(game, member).mhp;
        member.level += 1;
        let after = stats_of(game, member);
        member.hp = (member.hp + after.mhp - before).clamp(0, after.mhp);
        member.mp = member.mp.min(after.mmp);
        notes.push(format!("{} is now level {}!", actor.name, member.level));
        for learning in &class.learnings {
            if learning.level == member.level {
                if let Some(skill) = game.skill(learning.skill_id) {
                    notes.push(format!("Learned {}!", skill.name));
                }
            }
        }
    }
    notes
}

/// Spread `base` by `variance` percent. `roll` is 0..=100.
pub fn vary(base: i32, variance: i32, roll: i32) -> i32 {
    let base = base.max(0);
    let variance = variance.clamp(0, 100);
    let min = base * (100 - variance) / 100;
    let max = base * (100 + variance) / 100;
    if max <= min {
        return min;
    }
    min + (max - min) * roll.clamp(0, 100) / 100
}

pub fn scaled_amount(damage: &Damage, user: Stats, target: Stats, roll: i32) -> i32 {
    let base = match (damage.kind, damage.element) {
        (DamageKind::HpDamage | DamageKind::MpDamage, Element::Physical) => {
            damage.power + user.atk * 4 - target.def * 2
        }
        (DamageKind::HpDamage | DamageKind::MpDamage, Element::Magical) => {
            damage.power + user.mat * 4 - target.mdf * 2
        }
        (DamageKind::HpRecover | DamageKind::MpRecover, Element::Magical) => {
            damage.power + user.mat * 2
        }
        (DamageKind::HpRecover | DamageKind::MpRecover, Element::Physical) => damage.power,
        (DamageKind::None, _) => 0,
    };
    vary(base, damage.variance, roll)
}

pub fn basic_attack() -> Damage {
    Damage {
        kind: DamageKind::HpDamage,
        element: Element::Physical,
        power: 0,
        variance: 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fablespace_data::sample;

    #[test]
    fn stick_and_shirt_change_larks_stats() {
        let (game, _) = sample::fable_hollow();
        let member = member_from_actor(&game, 1).unwrap();
        let stats = stats_of(&game, &member);
        assert_eq!(stats.mhp, 48);
        assert_eq!(stats.atk, 7);
        assert_eq!(stats.def, 8);
        assert_eq!(stats.mat, 7);
        let roll = scaled_amount(
            &basic_attack(),
            stats,
            Stats {
                def: 6,
                ..Stats::default()
            },
            0,
        );
        assert_eq!(roll, 7 * 4 - 6 * 2);
    }
}
