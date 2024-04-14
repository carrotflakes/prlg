use std::{collections::HashMap, rc::Rc};

use crate::{data::Data, rule_map::RuleMap, runtime::Runtime, user_data::UserData};

pub struct Rule {
    pub head: Data,
    pub body: Vec<Data>,
    pub var_num: usize,
}

impl Rule {
    pub fn from_user_data(
        v: &[UserData],
        intern: &mut impl FnMut(Rc<String>) -> Rc<String>,
    ) -> Self {
        let mut scope = VariableScope::new();
        let mut it = v.iter().map(|x| scope.new_data(x, intern));
        let head = it.next().unwrap();
        let body = it.rev().collect::<Vec<_>>();
        Rule {
            var_num: scope.size(),
            head,
            body,
        }
    }
}

pub struct SymbolPool(HashMap<Rc<String>, Rc<String>>);

impl SymbolPool {
    pub fn new() -> Self {
        SymbolPool(Default::default())
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

    pub fn size(&self) -> usize {
        self.0.len()
    }

    pub fn new_data(
        &mut self,
        data: &UserData,
        intern: &mut impl FnMut(Rc<String>) -> Rc<String>,
    ) -> Data {
        let n = self.size();
        match data {
            UserData::Variable(v) => self
                .0
                .entry(v.clone())
                .or_insert_with(|| Data::Variable(n))
                .clone(),
            UserData::Wildcard => {
                let data = Data::Variable(n);
                self.0.insert(format!("unnamed:{}", n), data.clone());
                data
            }
            UserData::Term(v) => Data::Term(v.iter().map(|x| self.new_data(x, intern)).collect()),
            UserData::Symbol(s) => Data::Symbol(intern(Rc::new(s.clone()))),
        }
    }

    pub fn new_data_vec(
        &mut self,
        slice: &[UserData],
        intern: &mut impl FnMut(Rc<String>) -> Rc<String>,
    ) -> Vec<Data> {
        slice.iter().map(|x| self.new_data(x, intern)).collect()
    }
}

pub struct World {
    pub rules: Vec<Rule>,
    pub symbol_pool: SymbolPool,
    pub(crate) rule_map: RuleMap,
}

impl World {
    pub fn new(rules: Vec<Vec<UserData>>) -> Self {
        let mut symbol_pool = SymbolPool::new();
        let rules = rules
            .into_iter()
            .map(|rule| Rule::from_user_data(&rule, &mut |s| symbol_pool.get_and_insert(s)))
            .collect();
        Self {
            rule_map: RuleMap::from_rules(&rules),
            rules,
            symbol_pool,
        }
    }

    pub fn run<F: FnMut(&[Data])>(&self, data_slice: &[UserData], resolved_fn: F) {
        let goals = VariableScope::new().new_data_vec(data_slice, &mut |s| self.symbol_pool.get(s));
        Runtime::run(&self, &goals, resolved_fn);
    }
}
