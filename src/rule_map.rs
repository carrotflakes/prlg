use std::collections::HashMap;

use crate::{bindings::Bindings, data::Data, world::Rule};

pub struct RuleMap {
    map: HashMap<*const Data, Vec<usize>>,
    all: Vec<usize>,
}

impl RuleMap {
    pub fn new() -> Self {
        RuleMap {
            map: Default::default(),
            all: Vec::new(),
        }
    }

    pub fn from_rules(rules: &Vec<Rule>) -> Self {
        let mut map = HashMap::<*const Data, Vec<usize>>::new();
        for rule in rules {
            for sub_goal in rule.body.iter() {
                let key = sub_goal as *const Data;
                let mut rule_indices = Vec::new();
                for (i, rule) in rules.iter().enumerate() {
                    if unify(sub_goal, &rule.head) {
                        rule_indices.push(i);
                    }
                }
                map.insert(key, rule_indices);
            }
        }
        RuleMap {
            map,
            all: (0..rules.len()).collect(),
        }
    }

    #[inline]
    pub fn get(&self, data: *const Data) -> &[usize] {
        self.map.get(&data).unwrap_or(&self.all)
    }
}

fn unify(left: &Data, right: &Data) -> bool {
    let mut bindings = Bindings::new();
    bindings.push(left.max_var());
    let left = bindings.instance(left);
    bindings.push(right.max_var());
    let right = bindings.instance(right);
    bindings.unify(left, right)
}
