// A generic tree datatype.
//
// TODO: Use traits.

type fields<T> = {
    mut parent: option<T>,
    mut first_child: option<T>,
    mut last_child: option<T>,
    mut prev_sibling: option<T>,
    mut next_sibling: option<T>
};

iface rd_tree_ops<T> {
    fn with_tree_fields<R>(T, f: fn(fields<T>) -> R) -> R;
}

iface wr_tree_ops<T> {
    fn with_tree_fields<R>(T, f: fn(fields<T>) -> R) -> R;
}

fn each_child<T:copy,O:rd_tree_ops<T>>(
    ops: O, node: T, f: fn(T) -> bool) {

    let mut p = ops.with_tree_fields(node) { |f| f.first_child };
    loop {
        alt copy p {
          none { ret; }
          some(c) {
            if !f(c) { ret; }
            p = ops.with_tree_fields(c) { |f| f.next_sibling };
          }
        }
    }
}

fn empty<T>() -> fields<T> {
    {mut parent: none,
     mut first_child: none,
     mut last_child: none,
     mut prev_sibling: none,
     mut next_sibling: none}
}

fn add_child<T:copy,O:wr_tree_ops<T>>(
    ops: O, parent: T, child: T) {

    ops.with_tree_fields(child) { |child_tf|
        alt child_tf.parent {
          some(_) { fail "Already has a parent"; }
          none { child_tf.parent = some(parent); }
        }

        assert child_tf.prev_sibling == none;
        assert child_tf.next_sibling == none;

        ops.with_tree_fields(parent) { |parent_tf|
            alt copy parent_tf.last_child {
              none {
                parent_tf.first_child = some(child);
              }
              some(lc) {
                let lc = lc; // satisfy alias checker
                ops.with_tree_fields(lc) { |lc_tf|
                    assert lc_tf.next_sibling == none;
                    lc_tf.next_sibling = some(child);
                }
                child_tf.prev_sibling = some(lc);
              }
            }

            parent_tf.last_child = some(child);
        }
    }
}

fn get_parent<T:copy,O:rd_tree_ops<T>>(ops: O, node: T) -> option<T> {
    ops.with_tree_fields(node) { |tf| tf.parent }
}

#[cfg(test)]
mod test {
    enum dummy = @{
        fields: fields<dummy>,
        value: uint
    };

    enum dtree { dtree }

    impl of rd_tree_ops<dummy> for dtree {
        fn with_tree_fields<R>(d: dummy, f: fn(fields<dummy>) -> R) -> R {
            f(d.fields)
        }
    }

    impl of wr_tree_ops<dummy> for dtree {
        fn with_tree_fields<R>(d: dummy, f: fn(fields<dummy>) -> R) -> R {
            f(d.fields)
        }
    }

    fn new_dummy(v: uint) -> dummy {
        dummy(@{fields: empty(), value: v})
    }

    fn parent_with_3_children() -> {p: dummy, children: [dummy]} {
        let children = [new_dummy(0u),
                        new_dummy(1u),
                        new_dummy(2u)];
        let p = new_dummy(3u);

        for vec::each(children) {|c|
            add_child(dtree, p, c);
        }

        ret {p: p, children: children};
    }

    #[test]
    fn add_child_0() {
        let {p, children} = parent_with_3_children();
        let mut i = 0u;
        for each_child(dtree, p) {|c|
            assert c.value == i;
            i += 1u;
        }
        assert i == children.len();
    }

    #[test]
    fn add_child_break() {
        let {p, _} = parent_with_3_children();
        let mut i = 0u;
        for each_child(dtree, p) {|_c|
            i += 1u;
            break;
        }
        assert i == 1u;
    }
}
