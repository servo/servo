type fields<T> = @{
    mut parent: option<T>,
    mut first_child: option<T>,
    mut last_child: option<T>,
    mut prev_sibling: option<T>,
    mut next_sibling: option<T>
};

iface tree {
    fn tree_fields() -> fields<self>;
}

fn each_child<T:copy tree>(
    node: T, f: fn(T) -> bool) {

    let mut p = node.tree_fields().first_child;
    loop {
        alt p {
          none { ret; }
          some(c) {
            if !f(c) { ret; }
            p = c.tree_fields().next_sibling;
          }
        }
    }
}

fn empty<T>() -> fields<T> {
    @{mut parent: none,
      mut first_child: none,
      mut last_child: none,
      mut prev_sibling: none,
      mut next_sibling: none}
}

fn add_child<T:copy tree>(
    node: T, child: T) {

    let child_tf = child.tree_fields();
    alt child_tf.parent {
      some(_) { fail "Already has a parent"; }
      none { child_tf.parent = some(node); }
    }

    assert child_tf.prev_sibling == none;
    assert child_tf.next_sibling == none;

    let node_tf = node.tree_fields();
    alt node_tf.last_child {
      none {
        node_tf.first_child = some(child);
      }

      some(lc) {
        let lc_tf = lc.tree_fields();
        assert lc_tf.next_sibling == none;
        lc_tf.next_sibling = some(child);
        child_tf.prev_sibling = some(lc);
      }
    }

    node_tf.last_child = some(child);
}

#[cfg(test)]
mod test {
    enum dummy = @{
        fields: fields<dummy>,
        value: uint
    };

    impl of tree for dummy {
        fn tree_fields() -> fields<dummy> { self.fields }
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
            add_child(p, c);
        }

        ret {p: p, children: children};
    }

    #[test]
    fn add_child_0() {
        let {p, children} = parent_with_3_children();
        let mut i = 0u;
        for each_child(p) {|c|
            assert c.value == i;
            i += 1u;
        }
        assert i == children.len();
    }

    #[test]
    fn add_child_break() {
        let {p, _} = parent_with_3_children();
        let mut i = 0u;
        for each_child(p) {|_c|
            i += 1u;
            break;
        }
        assert i == 1u;
    }
}