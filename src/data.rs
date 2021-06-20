use std::rc::Rc;

#[derive(Debug, Clone)]
pub enum Data {
    Variable(usize),
    Symbol(Rc<String>),
    Term(Vec<Data>),
}

impl Data {
    pub(crate) fn max_var(&self) -> usize {
        match self {
            Data::Variable(n) => *n,
            Data::Symbol(_) => 0,
            Data::Term(v) => v.iter().map(|x| x.max_var() + 1).max().unwrap_or(0),
        }
    }

    #[inline]
    pub(crate) fn get_ref(&self) -> &'static Data {
        unsafe { std::mem::transmute::<_, &'static Data>(self) }
    }
}

impl std::fmt::Display for Data {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Data::Variable(n) => write!(f, "{{{}}}", n),
            Data::Symbol(s) => write!(f, "{}", s),
            Data::Term(v) => {
                write!(f, "(")?;
                if let Some(d) = v.first() {
                    write!(f, "{}", d)?;
                }
                for d in &v[1..] {
                    write!(f, " {}", d)?;
                }
                write!(f, ")")
            }
        }
    }
}
