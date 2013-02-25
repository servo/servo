use core::comm::{Chan, Port};

pub fn spawn_listener<A: Owned>(f: ~fn(Port<A>)) -> Chan<A> {
    let (setup_po, setup_ch) = comm::stream();
    do task::spawn {
        let (po, ch) = comm::stream();
        setup_ch.send(ch);
        f(po);
    }
    setup_po.recv()
}
