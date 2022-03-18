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
                if v.first().map(|d| if let Data::Symbol(s) = d {s.as_str() == "cons" } else {false}).unwrap_or(false) {
                    fn g(f: &mut std::fmt::Formatter<'_>, d: &Data, first: bool) -> std::fmt::Result {
                        if let Data::Symbol(s) = d {
                            if s.as_str() == "nil" {
                                return write!(f, "]");
                            }
                        }
                        if !first {
                            write!(f, " ")?;
                        }
                        if let Data::Term(v) = d {
                            if let Some(Data::Symbol(s)) = v.first() {
                                if v.len() == 3 && s.as_str() == "cons" {
                                    write!(f, "{}", &v[1])?;
                                    return g(f, &v[2], false);
                                }
                            }
                        }
                        write!(f, ". {}]", d)
                    }
                    write!(f, "[")?;
                    return g(f, self, true);
                }
                write!(f, "(")?;
                if let Some(d) = v.first() {
                    write!(f, "{}", d)?;
                }
                for d in &v[1..] { // TODO: check length
                    write!(f, " {}", d)?;
                }
                write!(f, ")")
            }
        }
    }
}
