use crate::{
    bindings::{Bindings, Instance},
    data::Data,
    world::World,
};

pub struct Context<'a> {
    world: &'a World,
    initial_goals: Vec<Data>,
    goals: Vec<Instance>,
    bindings: Bindings,
}

impl<'a> Context<'a> {
    pub fn new(world: &'a World, initial_goals: Vec<Data>) -> Self {
        let offset = initial_goals.iter().map(|d| d.max_var()).max().unwrap_or(0);
        let goals = initial_goals
            .iter()
            .map(|g| Instance::new(g.get_ref(), 0))
            .collect::<Vec<_>>();
        Self {
            world,
            bindings: Bindings::new(vec![None; offset], Vec::new()),
            goals,
            initial_goals,
        }
    }

    pub fn run<F: Fn(&Self)>(&mut self, f: &F) {
        if let Some(goal) = self.goals.pop() {
            let (bindings_len, indices_len) = self.bindings.check();
            let goal_num = self.goals.len();

            for rule in &self.world.rules {
                let head = Instance::new(&rule.head, bindings_len);
                self.bindings.alloc(bindings_len + rule.head_var_num);
                if self.bindings.unify(goal.clone(), head) {
                    self.goals
                        .extend(rule.body.iter().map(|d| Instance::new(d, bindings_len)));
                    self.bindings.alloc(bindings_len + rule.body_var_num);
                    self.run(f);
                }
                self.bindings.rewind(indices_len);
                self.goals.truncate(goal_num);
            }

            self.bindings.rewind_bindings(bindings_len);
            self.goals.push(goal);
        } else {
            f(self);
        }
    }

    pub fn print(&self) {
        for g in &self.initial_goals {
            println!("{}", self.bindings.instant(Instance::new(g.get_ref(), 0)));
        }
    }
}
