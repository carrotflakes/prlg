use std::io::{stdin, Stdin};

use crate::{bindings::Bindings, data::Data, World};

pub struct InteractiveRuntime<'a> {
    world: &'a World,
}

impl<'a> InteractiveRuntime<'a> {
    pub fn new(world: &'a World) -> Self {
        Self { world }
    }

    pub fn run(&mut self, mut goals: Vec<Data>) {
        let mut stdin = stdin();
        if goals.is_empty() {
            println!("goal!!!");
            return;
        }

        println!("=== select goal");
        for (i, goal) in goals.iter().enumerate() {
            println!("{:>4}: {}", i, &goal);
        }
        let n = get_number(&mut stdin, goals.len());

        let max_var = goals.iter().map(|d| d.max_var()).max().unwrap_or(0);
        let goal = goals.remove(n);
        let mut bindings = Bindings::new();
        bindings.push(max_var);
        let left = bindings.instance(&goal);
        let rest_goals: Vec<_> = goals.iter().map(|d| bindings.instance(d)).collect();
        let mut candidates: Vec<(Vec<Data>, Vec<Data>)> = self
            .world
            .rules
            .iter()
            .filter_map(|rule| {
                bindings.push(rule.var_num);
                let right = bindings.instance(&rule.head);
                if bindings.unify(left.clone(), right) {
                    let body: Vec<_> = rule.body.iter().map(|d| bindings.instance(d)).collect();
                    let subgoals: Vec<_> =
                        body.into_iter().rev().map(|i| bindings.data(i)).collect();
                    let rest_goals = rest_goals
                        .iter()
                        .map(|i| bindings.data(i.clone()))
                        .collect();
                    Some((subgoals, rest_goals))
                } else {
                    None
                }
            })
            .collect();

        println!("=== select candidate");
        for (i, c) in candidates.iter().enumerate() {
            println!("{:>4}: {} ... {}", i, &goal, goals.len());
            for d in c.0.iter() {
                println!("    - {}", d);
            }
        }
        let n = get_number(&mut stdin, candidates.len());
        let (subgoals, rest_goals) = candidates.remove(n);
        self.run(subgoals.into_iter().chain(rest_goals.into_iter()).collect())
    }
}

fn get_number(stdin: &mut Stdin, max: usize) -> usize {
    if max == 1 {
        return 0;
    }
    println!("number?");
    let mut buf = String::new();
    stdin.read_line(&mut buf).unwrap();
    if let Ok(n) = buf.trim().parse::<usize>() {
        if n < max {
            return n;
        }
    }
    get_number(stdin, max)
}
