use crate::{
    bindings::{Binder, Bindings, Instance},
    data::Data,
    world::World,
};

pub struct Runtime<'a, F: FnMut(&[Data])> {
    initial_goals: &'a [Data],
    resolved_fn: F,
    goals: Vec<Instance>,
    cut: std::rc::Rc<String>,
}

impl<'a, F: FnMut(&[Data])> Runtime<'a, F> {
    pub fn run(world: &World, initial_goals: &'a [Data], resolved_fn: F) {
        let var_num = initial_goals.iter().map(|d| d.max_var()).max().unwrap_or(0);
        let mut bindings = Bindings::new();
        let mut binder = bindings.binder(var_num);
        let goals = initial_goals
            .iter()
            .rev()
            .map(|d| binder.instance(d))
            .collect();
        Self {
            initial_goals,
            resolved_fn,
            goals,
            cut: world.symbol_pool.get(std::rc::Rc::new("cut".to_owned())),
        }
        .step(world, &mut binder);
    }

    fn step(&mut self, world: &World, binder: &mut Binder) -> bool {
        if let Some(goal) = self.goals.pop() {
            let goal_num = self.goals.len();

            if goal.data().as_symbol() == Some(&self.cut) {
                self.step(world, binder);
                self.goals.push(goal);
                return true;
            }

            for &rule_index in world.rule_map.get(goal.data()) {
                let rule = &world.rules[rule_index];
                let mut binder = binder.child(rule.var_num);
                let head = binder.instance(&rule.head);
                if binder.unify(goal.clone(), head) {
                    self.goals
                        .extend(rule.body.iter().map(|d| binder.instance(d)));
                    if self.step(world, &mut binder) {
                        return true;
                    }
                    self.goals.truncate(goal_num);
                }
            }

            self.goals.push(goal);
        } else {
            // All goals are resolved
            let datas: Vec<_> = self
                .initial_goals
                .iter()
                .map(|d| binder.data(Instance::new(d, 0)))
                .collect();
            (self.resolved_fn)(&datas);
        }
        false
    }
}
