// A generic tree datatype.
//
// TODO: Use traits.

type Tree<T> = {
    mut parent: Option<T>,
    mut first_child: Option<T>,
    mut last_child: Option<T>,
    mut prev_sibling: Option<T>,
    mut next_sibling: Option<T>
};

trait ReadMethods<T> {
    fn with_tree_fields<R>(T, f: fn(Tree<T>) -> R) -> R;
}

trait WriteMethods<T> {
    fn with_tree_fields<R>(T, f: fn(Tree<T>) -> R) -> R;
}

fn each_child<T:copy,O:ReadMethods<T>>(ops: O, node: T, f: fn(T) -> bool) {
    let mut p = ops.with_tree_fields(node, |f| f.first_child);
    loop {
        match copy p {
          None => { return; }
          Some(c) => {
            if !f(c) { return; }
            p = ops.with_tree_fields(c, |f| f.next_sibling);
          }
        }
    }
}

fn empty<T>() -> Tree<T> {
    {mut parent: None,
     mut first_child: None,
     mut last_child: None,
     mut prev_sibling: None,
     mut next_sibling: None}
}

fn add_child<T:copy,O:WriteMethods<T>>(ops: O, parent: T, child: T) {

    ops.with_tree_fields(child, |child_tf| {
        match child_tf.parent {
          Some(_) => { fail ~"Already has a parent"; }
          None => { child_tf.parent = Some(parent); }
        }

        assert child_tf.prev_sibling.is_none();
        assert child_tf.next_sibling.is_none();

        ops.with_tree_fields(parent, |parent_tf| {
            match copy parent_tf.last_child {
              None => {
                parent_tf.first_child = Some(child);
              }
              Some(lc) => {
                let lc = lc; // satisfy alias checker
                ops.with_tree_fields(lc, |lc_tf| {
                    assert lc_tf.next_sibling.is_none();
                    lc_tf.next_sibling = Some(child);
                });
                child_tf.prev_sibling = Some(lc);
              }
            }

            parent_tf.last_child = Some(child);
        });
    });
}

fn get_parent<T:copy,O:ReadMethods<T>>(ops: O, node: T) -> Option<T> {
    ops.with_tree_fields(node, |tf| tf.parent)
}

#[cfg(test)]
mod test {
    enum dummy = @{
        fields: Tree<dummy>,
        value: uint
    };

    enum dtree { dtree }

    impl dtree : ReadMethods<dummy> {
        fn with_tree_fields<R>(d: dummy, f: fn(Tree<dummy>) -> R) -> R {
            f(d.fields)
        }
    }

    impl dtree : WriteMethods<dummy> {
        fn with_tree_fields<R>(d: dummy, f: fn(Tree<dummy>) -> R) -> R {
            f(d.fields)
        }
    }

    fn new_dummy(v: uint) -> dummy {
        dummy(@{fields: empty(), value: v})
    }

    fn parent_with_3_children() -> {p: dummy, children: ~[dummy]} {
        let children = ~[new_dummy(0u),
                         new_dummy(1u),
                         new_dummy(2u)];
        let p = new_dummy(3u);

        for vec::each(children) |c| {
            add_child(dtree, p, c);
        }

        return {p: p, children: children};
    }

    #[test]
    fn add_child_0() {
        let {p, children} = parent_with_3_children();
        let mut i = 0u;
        for each_child(dtree, p) |c| {
            assert c.value == i;
            i += 1u;
        }
        assert i == children.len();
    }

    #[test]
    fn add_child_break() {
        let {p, _} = parent_with_3_children();
        let mut i = 0u;
        for each_child(dtree, p) |_c| {
            i += 1u;
            break;
        }
        assert i == 1u;
    }
}
