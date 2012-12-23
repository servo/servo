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
