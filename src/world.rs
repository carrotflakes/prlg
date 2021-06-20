use std::{collections::HashMap, rc::Rc};

use crate::{context::Context, data::Data, user_data::UserData};

pub struct Rule {
    pub head: Data,
    pub body: Vec<Data>,
    pub head_var_num: usize,
    pub body_var_num: usize,
}

pub struct SymbolScope(HashMap<Rc<String>, Rc<String>>);

impl SymbolScope {
    pub fn new() -> Self {
        SymbolScope(Default::default())
    }

    pub fn get_and_insert(&mut self, string: Rc<String>) -> Rc<String> {
        if let Some(r) = self.0.get(&string) {
            r.clone()
        } else {
            self.0.insert(string.clone(), string.clone());
            string
        }
    }

    pub fn get(&self, string: Rc<String>) -> Rc<String> {
        if let Some(r) = self.0.get(&string) {
            r.clone()
        } else {
            string
        }
    }
}

pub struct VariableScope(HashMap<String, Data>);

impl VariableScope {
    pub fn new() -> Self {
        VariableScope(Default::default())
    }

    pub fn new_data(
        &mut self,
        data: &UserData,
        get_str: &mut impl FnMut(Rc<String>) -> Rc<String>,
    ) -> Data {
        let n = self.0.len();
        match data {
            UserData::Variable(v) => match self.0.entry(v.clone()) {
                std::collections::hash_map::Entry::Occupied(data) => data.get().clone(),
                std::collections::hash_map::Entry::Vacant(e) => {
                    let data = Data::Variable(n);
                    e.insert(data.clone());
                    data
                }
            },
            UserData::Wildcard => {
                let data = Data::Variable(n);
                self.0.insert(format!("unnamed:{}", n), data.clone());
                data
            }
            UserData::Term(v) => Data::Term(v.iter().map(|x| self.new_data(x, get_str)).collect()),
            UserData::Symbol(s) => Data::Symbol(get_str(Rc::new(s.clone()))),
        }
    }

    pub fn new_data_vec(
        &mut self,
        slice: &[UserData],
        get_str: &mut impl FnMut(Rc<String>) -> Rc<String>,
    ) -> Vec<Data> {
        slice.iter().map(|x| self.new_data(x, get_str)).collect()
    }
}

pub struct World {
    pub rules: Vec<Rule>,
    pub symbol_scope: SymbolScope,
}

impl World {
    pub fn new(rules: Vec<Vec<UserData>>) -> Self {
        let mut world = Self {
            rules: vec![],
            symbol_scope: SymbolScope::new(),
        };
        for rule in rules {
            let rule = World::make_rule(rule, &mut |s| world.symbol_scope.get_and_insert(s));
            world.rules.push(rule);
        }
        world
    }

    pub fn run<F: Fn(&[Data])>(&self, data_slice: &[UserData], resolved_fn: &F) {
        let goals =
            VariableScope::new().new_data_vec(data_slice, &mut |s| self.symbol_scope.get(s));
        Context::run(&self, goals, resolved_fn);
    }

    fn make_rule(v: Vec<UserData>, get_str: &mut impl FnMut(Rc<String>) -> Rc<String>) -> Rule {
        let mut scope = VariableScope::new();
        let mut it = v.iter().map(|x| scope.new_data(x, get_str));
        let head = it.next().unwrap();
        let body = it.rev().collect::<Vec<_>>();
        let head_var_num = head.max_var();
        let body_var_num = body
            .iter()
            .map(|x| x.max_var())
            .max()
            .unwrap_or(head_var_num);
        Rule {
            head_var_num,
            body_var_num,
            head,
            body,
        }
    }
}
