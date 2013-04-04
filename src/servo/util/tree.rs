// A generic tree datatype.
//
// TODO: Use traits.

pub struct Tree<T> {
    parent: Option<T>,
    first_child: Option<T>,
    last_child: Option<T>,
    prev_sibling: Option<T>,
    next_sibling: Option<T>
}

pub trait ReadMethods<T> {
    fn with_tree_fields<R>(&self, &T, f: &fn(&mut Tree<T>) -> R) -> R;
}

pub trait WriteMethods<T> {
    fn with_tree_fields<R>(&self, &T, f: &fn(&mut Tree<T>) -> R) -> R;
    fn tree_eq(&self, &T, &T) -> bool;
}

pub fn each_child<T:Copy,O:ReadMethods<T>>(ops: &O, node: &T, f: &fn(&T) -> bool) {
    let mut p = ops.with_tree_fields(node, |f| f.first_child);
    loop {
        match copy p {
          None => { return; }
          Some(ref c) => {
            if !f(c) { return; }
            p = ops.with_tree_fields(c, |f| f.next_sibling);
          }
        }
    }
}

pub fn is_leaf<T:Copy,O:ReadMethods<T>>(ops: &O, node: &T) -> bool {
    first_child(ops, node).is_none()
}

pub fn first_child<T:Copy,O:ReadMethods<T>>(ops: &O, node: &T) -> Option<T> {
    ops.with_tree_fields(node, |tf| tf.first_child)
}

pub fn last_child<T:Copy,O:ReadMethods<T>>(ops: &O, node: &T) -> Option<T> {
    ops.with_tree_fields(node, |tf| tf.last_child)
}

pub fn next_sibling<T:Copy,O:ReadMethods<T>>(ops: &O, node: &T) -> Option<T> {
    ops.with_tree_fields(node, |tf| tf.next_sibling)
}

pub fn prev_sibling<T:Copy,O:ReadMethods<T>>(ops: &O, node: &T) -> Option<T> {
    ops.with_tree_fields(node, |tf| tf.prev_sibling)
}

pub fn parent<T:Copy,O:ReadMethods<T>>(ops: &O, node: &T) -> Option<T> {
    ops.with_tree_fields(node, |tf| tf.parent)
}

pub fn empty<T>() -> Tree<T> {
    Tree {
        mut parent: None,
        mut first_child: None,
        mut last_child: None,
        mut prev_sibling: None,
        mut next_sibling: None
    }
}

pub fn add_child<T:Copy,O:WriteMethods<T>>(ops: &O, parent: T, child: T) {
    assert!(!ops.tree_eq(&parent, &child));

    ops.with_tree_fields(&child, |child_tf| {
        match child_tf.parent {
          Some(_) => { fail!(~"Already has a parent"); }
          None => { child_tf.parent = Some(parent); }
        }

        assert!(child_tf.prev_sibling.is_none());
        assert!(child_tf.next_sibling.is_none());

        ops.with_tree_fields(&parent, |parent_tf| {
            match copy parent_tf.last_child {
              None => {
                parent_tf.first_child = Some(child);
              }
              Some(lc) => {
                ops.with_tree_fields(&lc, |lc_tf| {
                    assert!(lc_tf.next_sibling.is_none());
                    lc_tf.next_sibling = Some(child);
                });
                child_tf.prev_sibling = Some(lc);
              }
            }

            parent_tf.last_child = Some(child);
        });
    });
}

pub fn remove_child<T:Copy,O:WriteMethods<T>>(ops: &O, parent: T, child: T) {
    do ops.with_tree_fields(&child) |child_tf| {
        match copy child_tf.parent {
            None => { fail!(~"Not a child"); }
            Some(parent_n) => {
                assert!(ops.tree_eq(&parent, &parent_n));

                // adjust parent fields
                do ops.with_tree_fields(&parent) |parent_tf| {
                    match copy parent_tf.first_child {
                        None => { fail!(~"parent had no first child??") },
                        Some(first_child) if ops.tree_eq(&child, &first_child) => {
                            parent_tf.first_child = child_tf.next_sibling;
                        },
                        Some(_) => {}
                    };
                    
                    match copy parent_tf.last_child {
                        None => { fail!(~"parent had no last child??") },
                        Some(last_child) if ops.tree_eq(&child, &last_child) => {
                            parent_tf.last_child = child_tf.prev_sibling;
                        },
                        Some(_) => {}
                    }
                }
            }
        }

        // adjust siblings
        match child_tf.prev_sibling {
            None => {},
            Some(_) => {
                do ops.with_tree_fields(&child_tf.prev_sibling.get()) |prev_tf| {
                    prev_tf.next_sibling = child_tf.next_sibling;
                }
            }
        }
        match child_tf.next_sibling {
            None => {},
            Some(_) => {
                do ops.with_tree_fields(&child_tf.next_sibling.get()) |next_tf| {
                    next_tf.prev_sibling = child_tf.prev_sibling;
                }
            }
        }

        // clear child 
        child_tf.parent = None;
        child_tf.next_sibling = None;
        child_tf.prev_sibling = None;
    }
}

pub fn get_parent<T:Copy,O:ReadMethods<T>>(ops: &O, node: &T) -> Option<T> {
    ops.with_tree_fields(node, |tf| tf.parent)
}

#[cfg(test)]
mod test {
    use super::*;
    use core::managed::mut_ptr_eq;

    struct dummy {
        fields: Tree<@mut dummy>,
        value: uint
    }

    enum dtree { dtree }

    impl ReadMethods<@mut dummy> for dtree {
        fn with_tree_fields<R>(&self, d: &@mut dummy, f: &fn(&mut Tree<@mut dummy>) -> R) -> R {
            f(&mut d.fields)
        }
    }

    impl WriteMethods<@mut dummy> for dtree {
        fn with_tree_fields<R>(&self, d: &@mut dummy, f: &fn(&mut Tree<@mut dummy>) -> R) -> R {
            f(&mut d.fields)
        }
        fn tree_eq(&self, a: &@mut dummy, b: &@mut dummy) -> bool { mut_ptr_eq(*a, *b) }
    }

    fn new_dummy(v: uint) -> @mut dummy {
        @mut dummy {fields: empty(), value: v}
    }

    fn parent_with_3_children() -> (@mut dummy, ~[@mut dummy]) {
        let children = ~[new_dummy(0u),
                         new_dummy(1u),
                         new_dummy(2u)];
        let p = new_dummy(3u);

        for vec::each(children) |c| {
            add_child(&dtree, p, *c);
        }

        return (p, children);
    }

    #[test]
    fn add_child_0() {
        let (p, children) = parent_with_3_children();
        let mut i = 0u;
        for each_child(&dtree, &p) |c| {
            assert!(c.value == i);
            i += 1u;
        }
        assert!(i == children.len());
    }

    #[test]
    fn add_child_break() {
        let (p, _) = parent_with_3_children();
        let mut i = 0u;
        for each_child(&dtree, &p) |_c| {
            i += 1u;
            break;
        }
        assert!(i == 1u);
    }

    #[test]
    fn remove_first_child() {
        let (p, children) = parent_with_3_children();
        remove_child(&dtree, p, children[0]);

        let mut i = 0;
        for each_child(&dtree, &p) |_c| {
            i += 1;
        }
        assert!(i == 2);
    }

    #[test]
    fn remove_last_child() {
        let (p, children) = parent_with_3_children();
        remove_child(&dtree, p, children[2]);

        let mut i = 0;
        for each_child(&dtree, &p) |_c| {
            i += 1;
        }
        assert!(i == 2);
    }

    #[test]
    fn remove_middle_child() {
        let (p, children) = parent_with_3_children();
        remove_child(&dtree, p, children[1]);

        let mut i = 0;
        for each_child(&dtree, &p) |_c| {
            i += 1;
        }
        assert!(i == 2);
    }

    #[test]
    fn remove_all_child() {
        let (p, children) = parent_with_3_children();
        remove_child(&dtree, p, children[0]);
        remove_child(&dtree, p, children[1]);
        remove_child(&dtree, p, children[2]);

        let mut i = 0;
        for each_child(&dtree, &p) |_c| {
            i += 1;
        }
        assert!(i == 0);
    }
}
