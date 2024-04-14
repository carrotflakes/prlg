use std::rc::Rc;

use crate::data::Data;

#[derive(Debug, Clone)]
pub struct Instance {
    data: &'static Data,
    base: usize,
}

impl Instance {
    pub fn new(data: &Data, base: usize) -> Instance {
        Instance {
            data: data.get_ref(),
            base,
        }
    }

    pub fn data(&self) -> &Data {
        self.data
    }

    pub fn eq(&self, other: &Instance) -> bool {
        self.base == other.base && self.data as *const Data == other.data
    }
}

/// Bindings keeps bound variables and enables rewinding to a previous state.
pub struct Bindings {
    bindings: Vec<Option<Instance>>,
    indices: Vec<usize>,
}

impl Bindings {
    pub fn new() -> Self {
        Self {
            bindings: Vec::new(),
            indices: Vec::new(),
        }
    }

    pub fn binder<'a>(&'a mut self, size: usize) -> Binder<'a> {
        let bindings_len = self.bindings.len();
        let indices_len = self.indices.len();

        // Allocate
        self.bindings.resize(bindings_len + size, None);

        Binder {
            bindings_len,
            indices_len,
            bindings: self,
        }
    }

    pub fn size(&self) -> usize {
        self.bindings.len()
    }

    pub fn get_data(&self, idx: usize) -> Option<Data> {
        self.bindings[idx].as_ref().map(|i| self.data(i.clone()))
    }

    pub fn data(&self, instance: Instance) -> Data {
        match &instance.data {
            Data::Variable(n) => {
                if let Some(d) = &self.bindings[instance.base + n] {
                    self.data(d.clone())
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

    fn bind(&mut self, idx: usize, instance: Instance) {
        self.bindings[idx] = Some(instance);
        self.indices.push(idx);
    }

    fn rewind(&mut self, bindings_len: usize, indices_len: usize) {
        for idx in &self.indices[indices_len..] {
            self.bindings[*idx] = None;
        }
        self.indices.truncate(indices_len);
        self.bindings.truncate(bindings_len);
    }
}

pub struct Binder<'a> {
    bindings: &'a mut Bindings,
    bindings_len: usize,
    indices_len: usize,
}

impl<'a> Binder<'a> {
    pub fn child(&mut self, size: usize) -> Binder {
        self.bindings.binder(size)
    }

    pub fn bindings(&self) -> &Bindings {
        self.bindings
    }

    pub fn instance(&self, data: &Data) -> Instance {
        Instance::new(data, self.bindings_len)
    }

    pub fn data(&self, instance: Instance) -> Data {
        self.bindings.data(instance)
    }

    fn resolve(&self, mut instance: Instance) -> Instance {
        loop {
            if let Data::Variable(n) = instance.data {
                if let Some(d) = &self.bindings.bindings[instance.base + n] {
                    instance = d.clone();
                    continue;
                }
            }
            break;
        }
        instance
    }

    pub fn unify(&mut self, mut left: Instance, mut right: Instance) -> bool {
        left = self.resolve(left);
        right = self.resolve(right);

        if left.eq(&right) {
            return true;
        }

        match (&left.data, &right.data) {
            (Data::Variable(n), _) => {
                self.bindings.bind(left.base + n, right);
                true
            }
            (_, Data::Variable(n)) => {
                self.bindings.bind(right.base + n, left);
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
}

impl<'a> Drop for Binder<'a> {
    fn drop(&mut self) {
        self.bindings.rewind(self.bindings_len, self.indices_len);
    }
}
