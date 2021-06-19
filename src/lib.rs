pub mod macros;

use std::{collections::HashMap, rc::Rc};

#[derive(Debug)]
pub enum UserData {
    Variable(String),
    Wildcard,
    Symbol(String),
    Term(Vec<UserData>),
}

#[derive(Debug, Clone)]
pub enum Data {
    Variable(usize),
    Symbol(Rc<String>),
    Term(Vec<Data>),
}

impl Data {
    fn max_var(&self) -> usize {
        match self {
            Data::Variable(n) => *n,
            Data::Symbol(_) => 0,
            Data::Term(v) => v.iter().map(|x| x.max_var() + 1).max().unwrap_or(0),
        }
    }

    #[inline]
    fn get_ref(&self) -> &'static Data {
        unsafe { std::mem::transmute::<_, &'static Data>(self) }
    }
}

impl std::fmt::Display for Data {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Data::Variable(n) => write!(f, "{{{}}}", n),
            Data::Symbol(s) => write!(f, "{}", s),
            Data::Term(v) => {
                write!(f, "(")?;
                if let Some(d) = v.first() {
                    write!(f, "{}", d)?;
                }
                for d in &v[1..] {
                    write!(f, " {}", d)?;
                }
                write!(f, ")")
            }
        }
    }
}

pub struct Rule {
    head: Data,
    body: Vec<Data>,
    head_var_num: usize,
    body_var_num: usize,
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

    pub fn run<F: Fn(&Context)>(&self, data_slice: &[UserData], f: &F) {
        let goals =
            VariableScope::new().new_data_vec(data_slice, &mut |s| self.symbol_scope.get(s));
        Context::new(&self, goals).run(f);
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

#[derive(Clone)]
struct Instance {
    data: &'static Data,
    base: usize,
}

impl Instance {
    #[inline]
    fn new(data: &Data, base: usize) -> Instance {
        Instance {
            data: data.get_ref(),
            base,
        }
    }
}

pub struct Context<'a> {
    world: &'a World,
    initial_goals: Vec<Data>,
    bindings: Vec<Option<Instance>>,
    indices: Vec<usize>,
    goals: Vec<Instance>,
}

impl<'a> Context<'a> {
    pub fn new(world: &'a World, initial_goals: Vec<Data>) -> Self {
        let offset = initial_goals.iter().map(|d| d.max_var()).max().unwrap_or(0);
        let goals = initial_goals
            .iter()
            .map(|g| Instance {
                data: g.get_ref(),
                base: 0,
            })
            .collect::<Vec<_>>();
        Self {
            world,
            bindings: vec![None; offset],
            indices: Vec::new(),
            goals,
            initial_goals,
        }
    }

    pub fn run<F: Fn(&Self)>(&mut self, f: &F) {
        if let Some(goal) = self.goals.pop() {
            let bindings_len = self.bindings.len();
            let indices_len = self.indices.len();
            let goal_num = self.goals.len();

            for rule in &self.world.rules {
                let head = Instance::new(&rule.head, bindings_len);
                self.bindings.resize(bindings_len + rule.head_var_num, None);
                if self.unify(goal.clone(), head) {
                    self.goals
                        .extend(rule.body.iter().map(|d| Instance::new(d, bindings_len)));
                    self.bindings.resize(bindings_len + rule.body_var_num, None);
                    self.run(f);
                }
                self.rewind(indices_len);
                self.goals.truncate(goal_num);
            }

            self.bindings.truncate(bindings_len);
            self.goals.push(goal);
        } else {
            f(self);
        }
    }

    fn instant(&self, instance: Instance) -> Data {
        match &instance.data {
            Data::Variable(n) => {
                if let Some(d) = &self.bindings[instance.base + n] {
                    self.instant(d.clone())
                } else {
                    instance.data.clone()
                }
            }
            Data::Term(ds) => Data::Term(
                ds.iter()
                    .map(|d| self.instant(Instance::new(d, instance.base)))
                    .collect(),
            ),
            _ => instance.data.clone(),
        }
    }

    #[inline]
    fn resolve(&self, mut instance: Instance) -> Instance {
        loop {
            if let Data::Variable(n) = instance.data {
                if let Some(d) = &self.bindings[instance.base + n] {
                    instance = d.clone();
                    continue;
                }
            }
            break;
        }
        instance
    }

    fn unify(&mut self, mut left: Instance, mut right: Instance) -> bool {
        left = self.resolve(left);
        right = self.resolve(right);

        if left.base == right.base && left.data as *const Data == right.data {
            return true;
        }

        match (&left.data, &right.data) {
            (Data::Variable(n), _) => {
                self.bind(left.base + n, right);
                true
            }
            (_, Data::Variable(n)) => {
                self.bind(right.base + n, left);
                true
            }

            (Data::Symbol(l), Data::Symbol(r)) => Rc::ptr_eq(l, r),

            (Data::Term(l), Data::Term(r)) => {
                if l.len() != r.len() {
                    return false;
                }
                l.iter().zip(r.iter()).all(|(l, r)| {
                    self.unify(Instance::new(l, left.base), Instance::new(r, right.base))
                })
            }

            (Data::Symbol(_), Data::Term(_)) => false,
            (Data::Term(_), Data::Symbol(_)) => false,
        }
    }

    #[inline]
    fn bind(&mut self, var_n: usize, instance: Instance) {
        self.bindings[var_n] = Some(instance);
        self.indices.push(var_n);
    }

    #[inline]
    fn rewind(&mut self, len: usize) {
        for idx in &self.indices[len..] {
            self.bindings[*idx] = None;
        }
        self.indices.truncate(len);
    }

    pub fn print(&self) {
        for g in &self.initial_goals {
            println!(
                "{}",
                self.instant(Instance {
                    data: g.get_ref(),
                    base: 0
                })
            );
        }
    }
}
