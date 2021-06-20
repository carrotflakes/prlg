use std::rc::Rc;

use crate::data::Data;

#[derive(Clone)]
pub struct Instance {
    data: &'static Data,
    base: usize,
}

impl Instance {
    #[inline]
    pub fn new(data: &Data, base: usize) -> Instance {
        Instance {
            data: data.get_ref(),
            base,
        }
    }
}

pub struct Bindings {
    bindings: Vec<Option<Instance>>,
    indices: Vec<usize>,
}

impl Bindings {
    pub fn new(bindings: Vec<Option<Instance>>, indices: Vec<usize>) -> Self {
        Self { bindings, indices }
    }

    #[inline]
    pub fn check(&self) -> (usize, usize) {
        (self.bindings.len(), self.indices.len())
    }

    pub fn instant(&self, instance: Instance) -> Data {
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
    pub fn resolve(&self, mut instance: Instance) -> Instance {
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

    #[inline]
    pub fn unify(&mut self, mut left: Instance, mut right: Instance) -> bool {
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
    pub fn bind(&mut self, var_n: usize, instance: Instance) {
        self.bindings[var_n] = Some(instance);
        self.indices.push(var_n);
    }

    #[inline]
    pub fn alloc(&mut self, len: usize) {
        self.bindings.resize(len, None);
    }

    #[inline]
    pub fn rewind(&mut self, len: usize) {
        for idx in &self.indices[len..] {
            self.bindings[*idx] = None;
        }
        self.indices.truncate(len);
    }

    #[inline]
    pub fn rewind_bindings(&mut self, len: usize) {
        self.bindings.truncate(len);
    }
}
