use crate::{
    bindings::{Binder, Bindings, Instance},
    data::Data,
    world::World,
};

pub struct Runtime<'a, F: FnMut(&[Data])> {
    world: &'a World,
    initial_goals: Vec<Data>,
    resolved_fn: F,
}

impl<'a, F: FnMut(&[Data])> Runtime<'a, F> {
    pub fn run(world: &'a World, initial_goals: Vec<Data>, resolved_fn: F) {
        let base = initial_goals.iter().map(|d| d.max_var()).max().unwrap_or(0);
        let mut bindings = Bindings::new();
        let mut binder = bindings.binder();
        let mut goals = binder.instances(&initial_goals);
        binder.alloc(base);
        Self {
            world,
            initial_goals,
            resolved_fn,
        }
        .step(&mut goals, &mut binder);
    }

    #[inline]
    fn step(&mut self, goals: &mut Vec<Instance>, binder: &mut Binder) {
        if let Some(goal) = goals.pop() {
            let mut binder = binder.child();
            let goal_num = goals.len();

            for rule_index in self.get_rules(&goal) {
                // for rule_index in 0..self.world.rules.len() {
                let rule = &self.world.rules[rule_index];
                let head = binder.instance(&rule.head);
                binder.alloc(rule.head_var_num);
                if binder.unify(goal.clone(), head) {
                    goals.extend(binder.instances(&rule.body));
                    binder.alloc(rule.body_var_num);
                    self.step(goals, &mut binder);
                    goals.truncate(goal_num);
                }
                binder.rewind();
            }

            goals.push(goal);
        } else {
            let datas = self
                .initial_goals
                .iter()
                .map(|d| binder.data(Instance::new(d, 0)))
                .collect::<Vec<_>>();
            (self.resolved_fn)(datas.as_slice());
        }
    }

    fn get_rules(&self, goal: &Instance) -> Vec<usize> {
        self.world
            .rule_map
            .get(goal.data())
            .map(|x| x.to_vec())
            .unwrap_or_else(|| (0..self.world.rules.len()).collect())
    }
}
