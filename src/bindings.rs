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

    pub fn data(&self) -> &Data {
        self.data
    }
}

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

    pub fn binder<'a>(&'a mut self) -> Binder<'a> {
        Binder {
            bindings_len: self.bindings.len(),
            indices_len: self.indices.len(),
            bindings: self,
        }
    }
}

pub struct Binder<'a> {
    bindings: &'a mut Bindings,
    bindings_len: usize,
    indices_len: usize,
}

impl<'a> Binder<'a> {
    #[inline]
    pub fn child(&mut self) -> Binder {
        Binder {
            bindings_len: self.bindings.bindings.len(),
            indices_len: self.bindings.indices.len(),
            bindings: self.bindings,
        }
    }

    #[inline]
    pub fn instance(&self, data: &Data) -> Instance {
        Instance::new(data, self.bindings_len)
    }

    #[inline]
    pub fn instances(&self, data: &[Data]) -> Vec<Instance> {
        data.iter().map(|d| self.instance(d)).collect()
    }

    pub fn data(&self, instance: Instance) -> Data {
        match &instance.data {
            Data::Variable(n) => {
                if let Some(d) = &self.bindings.bindings[instance.base + n] {
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

    #[inline]
    pub fn resolve(&self, mut instance: Instance) -> Instance {
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
        self.bindings.bindings[var_n] = Some(instance);
        self.bindings.indices.push(var_n);
    }

    #[inline]
    pub fn alloc(&mut self, len: usize) {
        self.bindings.bindings.resize(self.bindings_len + len, None);
    }

    #[inline]
    pub fn dealloc(&mut self) {
        self.bindings.bindings.truncate(self.bindings_len);
    }

    #[inline]
    pub fn rewind(&mut self) {
        for idx in &self.bindings.indices[self.indices_len..] {
            self.bindings.bindings[*idx] = None;
        }
        self.bindings.indices.truncate(self.indices_len);
    }
}

impl<'a> Drop for Binder<'a> {
    #[inline]
    fn drop(&mut self) {
        self.dealloc()
    }
}
