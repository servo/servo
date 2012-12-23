use core::task;

pub fn spawn_listener<A: Owned>(
    +f: fn~(oldcomm::Port<A>)) -> oldcomm::Chan<A> {
    let setup_po = oldcomm::Port();
    let setup_ch = oldcomm::Chan(&setup_po);
    do task::spawn |move f| {
        let po = oldcomm::Port();
        let ch = oldcomm::Chan(&po);
        oldcomm::send(setup_ch, ch);
        f(move po);
    }
    oldcomm::recv(setup_po)
}

pub fn spawn_conversation<A: Owned, B: Owned>
    (+f: fn~(oldcomm::Port<A>, oldcomm::Chan<B>))
    -> (oldcomm::Port<B>, oldcomm::Chan<A>) {
    let from_child = oldcomm::Port();
    let to_parent = oldcomm::Chan(&from_child);
    let to_child = do spawn_listener |move f, from_parent| {
        f(from_parent, to_parent)
    };
    (from_child, to_child)
}
