use core::pipes::{Chan, Port};
use core::pipes;
use core::task;
use std::cell::Cell;

pub fn spawn_listener<A: Owned>(f: fn~(Port<A>)) -> Chan<A> {
    let (setup_po, setup_ch) = pipes::stream();
    do task::spawn {
        let (po, ch) = pipes::stream();
        setup_ch.send(ch);
        f(po);
    }
    setup_po.recv()
}

pub fn spawn_conversation<A: Owned, B: Owned>(f: fn~(Port<A>, Chan<B>)) -> (Port<B>, Chan<A>) {
    let (from_child, to_parent) = pipes::stream();
    let to_parent = Cell(to_parent);
    let to_child = do spawn_listener |from_parent| {
        f(from_parent, to_parent.take())
    };
    (from_child, to_child)
}
