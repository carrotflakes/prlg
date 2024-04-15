use crate::{
    bindings::{Bindings, Instance},
    data::Data,
    world::World,
};

pub struct Runtime<'a, F: FnMut(&[Data])> {
    initial_goals: Vec<Instance<'a>>,
    resolved_fn: F,
    goals: Vec<Instance<'a>>,
    bindings: Bindings<'a>,
    steps: Vec<Step<'a>>,
    cut: std::rc::Rc<String>,
}

struct Step<'a> {
    goal: Instance<'a>,
    goal_index: usize,
    rule_indices: std::slice::Iter<'a, usize>,
}

impl<'a, F: FnMut(&[Data])> Runtime<'a, F> {
    pub fn run(world: &'a World, goals: &'a [Data], resolved_fn: F) {
        let mut bindings = Bindings::new();
        let var_num = goals.iter().map(|d| d.max_var()).max().unwrap_or(0);
        bindings.push(var_num);
        let goals: Vec<_> = goals.iter().rev().map(|d| bindings.instance(d)).collect();

        let mut rt = Self {
            initial_goals: goals.clone(),
            resolved_fn,
            goals,
            bindings,
            steps: vec![],
            cut: world.symbol_pool.get(std::rc::Rc::new("cut".to_owned())),
        };
        rt.next_step(world);
        rt.process(world);
    }

    fn next_step(&mut self, world: &'a World) {
        while self.goals.last().and_then(|g| g.data().as_symbol()) == Some(&self.cut) {
            self.goals.pop();

            self.stop_backtrack();
        }

        if let Some(goal) = self.goals.pop() {
            self.steps.push(Step {
                rule_indices: world.rule_map.get(goal.data()).iter(),
                goal_index: self.goals.len(),
                goal,
            });
        } else {
            // All goals are resolved
            let datas: Vec<_> = self
                .initial_goals
                .iter()
                .map(|&i| self.bindings.data(i))
                .collect();
            (self.resolved_fn)(&datas);
            self.bindings.pop();
        }
    }

    fn process(&mut self, world: &'a World) {
        while let Some(step) = self.steps.last_mut() {
            let Some(&rule_index) = step.rule_indices.next() else {
                self.goals.truncate(step.goal_index);
                self.goals.push(step.goal);
                self.steps.pop();
                self.bindings.pop();
                continue;
            };

            let rule = &world.rules[rule_index];
            self.bindings.push(rule.var_num);
            let head = self.bindings.instance(&rule.head);
            if !self.bindings.unify(step.goal, head) {
                self.bindings.pop();
                continue;
            }

            self.goals.truncate(step.goal_index);
            self.goals
                .extend(rule.body.iter().map(|d| self.bindings.instance(d)));
            self.next_step(world);
        }
    }

    fn stop_backtrack(&mut self) {
        self.steps.clear();
    }
}
