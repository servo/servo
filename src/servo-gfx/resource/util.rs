use core::pipes::{Chan, Port};
use core::pipes;

pub fn spawn_listener<A: Owned>(f: ~fn(Port<A>)) -> Chan<A> {
    let (setup_po, setup_ch) = pipes::stream();
    do task::spawn |move f| {
        let (po, ch) = pipes::stream();
        setup_ch.send(ch);
        f(move po);
    }
    setup_po.recv()
}
