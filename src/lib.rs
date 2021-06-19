pub mod macros;

use std::{borrow::Cow, collections::HashMap, rc::Rc, usize};

#[derive(Debug)]
pub enum UserData {
    Variable(String),
    Wildcard,
    Symbol(String),
    Term(Vec<Rc<UserData>>),
}

#[derive(Debug)]
pub enum Data {
    Variable(usize),
    Symbol(String),
    Term(Vec<Rc<Data>>),
}

impl Data {
    fn max_var(&self) -> usize {
        match self {
            Data::Variable(n) => *n,
            Data::Symbol(_) => 0,
            Data::Term(v) => v.iter().map(|x| x.max_var() + 1).max().unwrap_or(0),
        }
    }
}

pub struct Rule {
    head: Rc<Data>,
    body: Vec<Rc<Data>>,
    head_var_num: usize,
    body_var_num: usize,
}

impl From<Vec<Rc<UserData>>> for Rule {
    fn from(v: Vec<Rc<UserData>>) -> Self {
        let mut scope = VariableScope::new();
        let mut it = v.iter().map(|x| scope.new_data(x));
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

pub struct VariableScope(HashMap<String, Rc<Data>>);

impl VariableScope {
    pub fn new() -> Self {
        VariableScope(Default::default())
    }

    pub fn new_data(&mut self, data: &Rc<UserData>) -> Rc<Data> {
        let n = self.0.len();
        match data.as_ref() {
            UserData::Variable(v) => match self.0.entry(v.clone()) {
                std::collections::hash_map::Entry::Occupied(data) => data.get().clone(),
                std::collections::hash_map::Entry::Vacant(e) => {
                    let data = Rc::new(Data::Variable(n));
                    e.insert(data.clone());
                    data
                }
            },
            &UserData::Wildcard => {
                let data = Rc::new(Data::Variable(n));
                self.0.insert(format!("unnamed:{}", n), data.clone());
                data
            },
            UserData::Term(v) => Rc::new(Data::Term(v.iter().map(|x| self.new_data(x)).collect())),
            UserData::Symbol(s) => Rc::new(Data::Symbol(s.clone())),
        }
    }

    pub fn new_data_vec(&mut self, slice: &[Rc<UserData>]) -> Vec<Rc<Data>> {
        slice.iter().map(|x| self.new_data(x)).collect()
    }
}

pub struct World {
    pub rules: Vec<Rule>,
}

impl World {
    pub fn new(rules: Vec<Vec<Rc<UserData>>>) -> Self {
        Self {
            rules: rules.into_iter().map(|x| x.into()).collect(),
        }
    }
}

pub struct Context<'a> {
    world: &'a World,
    bindings: Vec<Option<Rc<Data>>>,
    indices: Vec<usize>,
    goals: Vec<Rc<Data>>,
    initial_goals: Vec<Rc<Data>>,
}

impl<'a> Context<'a> {
    pub fn new(world: &'a World, goals: Vec<Rc<Data>>) -> Self {
        let offset = goals.iter().map(|d| d.max_var()).max().unwrap_or(0);
        Self {
            world,
            bindings: vec![None; offset],
            indices: Vec::new(),
            goals: goals.clone(),
            initial_goals: goals,
        }
    }

    pub fn run<F: Fn(&Self)>(&mut self, f: &F) {
        if let Some(goal) = self.goals.pop() {
            let bindings_len = self.bindings.len();
            let indices_len = self.indices.len();
            let goal_num = self.goals.len();

            for rule in &self.world.rules {
                let head = self.new_data(&rule.head, bindings_len);
                self.bindings.resize(bindings_len + rule.head_var_num, None);
                if self.unify(&goal, &head) {
                    for data in rule.body.iter() {
                        let data = self.new_data(data, bindings_len);
                        self.goals.push(data);
                    }
                    self.bindings.resize(bindings_len + rule.body_var_num, None);
                    self.run(f);
                }
                self.rewind(indices_len);
                self.goals.truncate(goal_num);
            }

            self.bindings.truncate(bindings_len);
            self.goals.push(goal);
        } else {
            // dbg!(&self.bindings, &self.indices);
            f(self);
        }
    }

    fn instant(&self, data: &Rc<Data>) -> Rc<Data> {
        match data.as_ref() {
            Data::Variable(n) => {
                if let Some(d) = &self.bindings[*n] {
                    self.instant(d)
                } else {
                    data.clone()
                }
            }
            Data::Term(ds) => Rc::new(Data::Term(ds.iter().map(|d| self.instant(d)).collect())),
            _ => data.clone(),
        }
    }

    fn unify(&mut self, left: &Rc<Data>, right: &Rc<Data>) -> bool {
        let mut left = Cow::Borrowed(left);
        loop {
            if let Data::Variable(n) = left.as_ref().as_ref() {
                if let Some(d) = &self.bindings[*n] {
                    left = Cow::Owned(d.clone());
                    continue;
                }
            }
            break;
        }
        let mut right = Cow::Borrowed(right);
        loop {
            if let Data::Variable(n) = right.as_ref().as_ref() {
                if let Some(d) = &self.bindings[*n] {
                    right = Cow::Owned(d.clone());
                    continue;
                }
            }
            break;
        }

        if Rc::ptr_eq(left.as_ref(), right.as_ref()) {
            return true;
        }

        match (left.as_ref().as_ref(), right.as_ref().as_ref()) {
            (Data::Variable(n), _) => {
                self.bind(*n, right.into_owned());
                true
            }
            (_, Data::Variable(n)) => {
                self.bind(*n, left.into_owned());
                true
            }

            (Data::Symbol(l), Data::Symbol(r)) => l == r,

            (Data::Term(l), Data::Term(r)) => {
                if l.len() != r.len() {
                    return false;
                }
                for (l, r) in l.iter().zip(r.iter()) {
                    if !self.unify(l, r) {
                        return false;
                    }
                }
                true
            }

            (Data::Symbol(_), Data::Term(_)) => false,
            (Data::Term(_), Data::Symbol(_)) => false,
        }
    }

    #[inline]
    fn bind(&mut self, var_n: usize, data: Rc<Data>) {
        self.bindings[var_n] = Some(data);
        self.indices.push(var_n);
    }

    #[inline]
    fn rewind(&mut self, len: usize) {
        for idx in &self.indices[len..] {
            self.bindings[*idx] = None;
        }
        self.indices.truncate(len);
    }

    #[inline]
    fn new_data(&mut self, data: &Rc<Data>, base_index: usize) -> Rc<Data> {
        match data.as_ref() {
            Data::Variable(n) => Rc::new(Data::Variable(base_index + n)),
            Data::Term(v) => Rc::new(Data::Term(
                v.iter().map(|x| self.new_data(x, base_index)).collect(),
            )),
            Data::Symbol(_) => data.clone(),
        }
    }

    pub fn print(&self) {
        for g in &self.initial_goals {
            println!("{:?}", self.instant(&g));
        }
    }
}
