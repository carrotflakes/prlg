#[macro_use]
extern crate prlg;

use prlg::{interactive_runtime::InteractiveRuntime, world::VariableScope, World};

fn main() {
    let rules = rules![
        (eq {x} {x})

        (member {x} (cons {x} {}))
        (member {x} (cons {} {xs})) {
            (member {x} {xs})
        }

        (nextto {x} {y} {list}) {
            (iright {x} {y} {list})
        }
        (nextto {x} {y} {list}) {
            (iright {y} {x} {list})
        }

        (iright {left} {right} (cons {left} (cons {right} {})))
        (iright {left} {right} (cons {} {rest})) {
            (iright {left} {right} {rest})
        }

        (zebra {h} {w} {z}) {
            (eq {h} [
                (house norwegian {} {} {} {})
                {}
                (house {} {} {} milk {})
                {}
                {}
            ])
            (member (house englishman {} {} {} red) {h})
            (member (house spaniard dog {} {} {}) {h})
            (member (house {} {} {} coffee green) {h})
            (member (house ukrainian {} {} tea {}) {h})
            (iright (house {} {} {} {} ivory)
                    (house {} {} {} {} green) {h})
            (member (house {} snails winston {} {}) {h})
            (member (house {} {} kools {} yellow) {h})
            (nextto (house {} {} chesterfield {} {})
                    (house {} fox {} {} {}) {h})
            (nextto (house {} {} kools {} {})
                    (house {} horse {} {} {}) {h})
            (member (house {} {} luckystrike oj {}) {h})
            (member (house japanese {} parliaments {} {}) {h})
            (nextto (house norwegian {} {} {} {})
                    (house {} {} {} {} blue) {h})
            (member (house {w} {} {} water {}) {h})
            (member (house {z} zebra {} {} {}) {h})
        }
    ];
    let world = World::new(rules);

    let data_slice = &[data! {(zebra {h} {w} {z})}];
    let goals = VariableScope::new().new_data_vec(data_slice, &mut |s| world.symbol_pool.get(s));

    InteractiveRuntime::new(&world).run(goals);
}
