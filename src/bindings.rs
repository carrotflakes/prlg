use std::rc::Rc;

use crate::data::Data;

#[derive(Debug, Clone, Copy)]
pub struct Instance<'a> {
    data: &'a Data,
    base: usize,
}

impl<'a> Instance<'a> {
    pub fn new(data: &'a Data, base: usize) -> Self {
        Instance { data, base }
    }

    pub fn data(&self) -> &Data {
        self.data
    }

    pub fn eq(&self, other: &Instance) -> bool {
        self.base == other.base && self.data as *const Data == other.data
    }
}

/// Bindings keeps bound variables and enables rewinding to a previous state.
pub struct Bindings<'a> {
    bindings: Vec<Option<Instance<'a>>>,
    indices: Vec<usize>,
    stack: Vec<(usize, usize)>,
}

impl<'a> Bindings<'a> {
    pub fn new() -> Self {
        Self {
            bindings: Vec::new(),
            indices: Vec::new(),
            stack: Vec::new(),
        }
    }

    pub fn push(&mut self, size: usize) {
        let bindings_len = self.bindings.len();
        self.stack.push((bindings_len, self.indices.len()));
        self.bindings.resize(bindings_len + size, None);
    }

    pub fn pop(&mut self) {
        if let Some((bindings_len, indices_len)) = self.stack.pop() {
            for idx in &self.indices[indices_len..] {
                self.bindings[*idx] = None;
            }
            self.indices.truncate(indices_len);
            self.bindings.truncate(bindings_len);
        }
    }

    pub fn instance(&self, data: &'a Data) -> Instance<'a> {
        Instance::new(
            data,
            self.stack
                .last()
                .map_or(0, |(bindings_len, _)| *bindings_len),
        )
    }

    pub fn unify(&mut self, mut left: Instance<'a>, mut right: Instance<'a>) -> bool {
        left = self.resolve(left);
        right = self.resolve(right);

        if left.eq(&right) {
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

    fn resolve(&self, mut instance: Instance<'a>) -> Instance<'a> {
        loop {
            if let Data::Variable(n) = instance.data {
                if let Some(i) = self.bindings[instance.base + n] {
                    instance = i;
                    continue;
                }
            }
            break;
        }
        instance
    }

    pub fn size(&self) -> usize {
        self.bindings.len()
    }

    pub fn get_data(&self, idx: usize) -> Option<Data> {
        self.bindings[idx].as_ref().map(|&i| self.data(i))
    }

    pub fn data(&self, instance: Instance<'a>) -> Data {
        match &instance.data {
            Data::Variable(n) => {
                if let Some(i) = self.bindings[instance.base + n] {
                    self.data(i)
                } else {
                    instance.data.clone()
                }
            }
            Data::Term(ds) => Data::Term(
                ds.iter()
                    .map(|d| self.data(Instance::new(d, instance.base)))
                    .collect(),
            ),
            _ => instance.data.clone(),
        }
    }

    fn bind(&mut self, idx: usize, instance: Instance<'a>) {
        self.bindings[idx] = Some(instance);
        self.indices.push(idx);
    }
}
