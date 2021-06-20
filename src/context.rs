use std::rc::Rc;

use crate::{data::Data, world::World};

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
