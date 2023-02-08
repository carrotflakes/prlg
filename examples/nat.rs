#[macro_use]
extern crate prlg;

use prlg::World;

fn main() {
    let rules = rules![
        (add zero {y} {y})
        (add (s {x}) {y} (s {z})) {
            (add {x} {y} {z})
        }

        (mul zero {} zero)
        (mul (s {x}) {y} {z}) {
            (mul {x} {y} {z_})
            (add {y} {z_} {z})
        }

        (le {x} {y}) {
            (add {x} {} {y})
        }
        (lt {x} (s {y})) {
            (add {x} {} {y})
        }

        (div {x} {y} zero {x}) {
            (lt {x} {y})
        }
        (div {x} {y} (s {q}) {r}) {
            (add {y} {x_} {x})
            (div {x_} {y} {q} {r})
        }
    ];

    // dbg!(&rules);
    let world = World::new(rules);

    world.run(&[data! {(add (s zero) (s (s zero)) {})}], |c| {
        for d in c {
            println!("{}", d)
        }
    });
    println!();
    world.run(
        &[data! {(mul (s (s (s zero))) (s (s (s (s zero)))) {})}],
        |c| {
            for d in c {
                println!("{}", d)
            }
        },
    );
    println!();
    world.run(
        &[
            data! {(mul (s (s (s zero))) (s (s (s (s zero)))) {a})}, // 3 * 4
            data! {(div (s {a}) (s (s (s zero))) {b} {c})},          // (12 + 1) / 3
        ],
        |c| {
            for d in c {
                println!("{}", d)
            }
        },
    );
    println!();
}
