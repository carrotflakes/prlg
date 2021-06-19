#[macro_export]
macro_rules! term {
    ($($e:expr),*) => {
        ::prlg::UserData::Term(vec![$($e),*])
    };
}
#[macro_export]
macro_rules! var {
    ($i:ident) => {
        ::prlg::UserData::Variable(stringify!($i).to_string())
    };
}
#[macro_export]
macro_rules! wild {
    () => {
        ::prlg::UserData::Wildcard
    };
}
#[macro_export]
macro_rules! sym {
    ($i:ident) => {
        ::prlg::UserData::Symbol(stringify!($i).to_string())
    };
}

#[macro_export]
macro_rules! data {
    ([]) => {
        data!{nil}
    };
    ([.$t:tt]) => {
        data!{$t}
    };
    ([$x:tt $($e:tt)*]) => {
        data!{(cons $x [$($e)*])}
    };
    (($($e:tt)*)) => {
        term![$(data!($e)),*]
    };
    ({$i:ident}) => {
        var!($i)
    };
    ({}) => {
        wild!()
    };
    ($i:ident) => {
        sym!($i)
    };
}

#[macro_export]
macro_rules! rules2 {
    ({} {$($t:tt),*}) => {
        vec![$($t),*]
    };
    ({$head:tt {$($t3:tt)*} $($t1:tt)*} {$($t2:tt),*}) => {
        rules2!({$($t1)*} {$($t2,)* (vec![data!($head), $(data!($t3)),*])})
    };
    ({$head:tt $($t1:tt)*} {$($t2:tt),*}) => {
        rules2!({$($t1)*} {$($t2,)* (vec![data!($head)])})
    };
}
#[macro_export]
macro_rules! rules {
    ($($t:tt)*) => {
        rules2!({$($t)*} {})
    };
}
