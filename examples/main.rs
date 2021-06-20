#[macro_use]
extern crate prlg;

use prlg::World;

fn main() {
    let rules = rules![
        (yo {x})
        (hoge fuga)
        (my_list [a b c])

        (member {x} [{x} . {}])
        (member {x} [{} . {xs}]) {
            (member {x} {xs})
        }

        (append nil {xs} {xs})
        (append [{x} . {xs}] {ys} [{x} . {zs}]) {
            (append {xs} {ys} {zs})
        }

        (isa ピカチュウ pokemon)
        (isa カイリュー pokemon)
        (isa ヤドラン pokemon)

        (all_pokemon {p}) {
            (isa {p} pokemon)
        }

        (hogehoge ({fuga} {fuga} {piyp})) {
            {piyo}
        }

        (eq {x} {x})

        (perm {xs}) {
            (eq {xs} [{} {} {}])
            (member a {xs})
            (member b {xs})
            (member c {xs})
        }
    ];

    // dbg!(&rules);
    let world = World::new(rules);
    // dbg!(&world.rules);

    world.run(&[data! {(all_pokemon {nyan})}],&|c| for d in c {println!("{}", d)});
    println!();
    world.run(&[data! {(append (cons a nil) (cons b nil) {nyan})}],&|c| for d in c {println!("{}", d)});
    println!();
    world.run(&[data! {(my_list {nyan})}],&|c| for d in c {println!("{}", d)});
    println!();
    world.run(&[data! {(perm {nyan})}],&|c| for d in c {println!("{}", d)});
    println!();
}
