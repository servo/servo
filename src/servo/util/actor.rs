/*!
An actor type
*/

use pipes::{Port, Chan, SharedChan, stream};

/**
The client reference to an actor

Actors are only referred to by opaque handles parameterized
over the actor's message type, which can be considered the
actor's interface.
*/
struct ActorRef<M: Send> {
    chan: Chan<M>,
}

impl<M: Send> ActorRef<M> {
    fn send(msg: M) {
        self.chan.send(move msg);
    }
}

/// The local actor interface
trait Actor<M> {
    fn handle(msg: M) -> bool;
}

/// A helper function used by actor constructors
fn spawn<A: Actor<M>, M: Send>(f: ~fn() -> A) -> ActorRef<M> {
    let (chan, port) = stream();
    do task::spawn |move f, move port| {
        let actor = f();
        loop {
            let msg = port.recv();
            if !actor.handle(move msg) {
                break;
            }
        }
    }

    return ActorRef {
        chan: move chan
    }
}

struct SharedActorRef<M: Send> {
    chan: SharedChan<M>
}

impl<M: Send> SharedActorRef<M> {
    fn send(msg: M) {
        self.chan.send(move msg);
    }
}

fn SharedActorRef<M: Send>(actor: ActorRef<M>) -> SharedActorRef<M> {
    let chan = match move actor {
        ActorRef {
            chan: move chan
        } => {
            move chan
        }
    };

    SharedActorRef {
        chan: SharedChan(move chan)
    }
}

#[cfg(test)]
mod test {

    enum HelloMsg {
        Exit(Chan<()>)
    }

    struct HelloActor {
        name: ~str
    }

    impl HelloActor: Actor<HelloMsg> {
        fn handle(msg: HelloMsg) -> bool {
            match msg {
                Exit(chan) => {
                    chan.send(());
                    return false;
                }
            }
        }
    }

    fn HelloActor(name: ~str) -> ActorRef<HelloMsg> {
        do spawn |move name| {
            HelloActor {
                name: copy name
            }
        }        
    }


    #[test]
    fn test_exit() {
        let actor = HelloActor(~"bob");
        let (chan, port) = stream();
        actor.send(Exit(move chan));
        port.recv();
    }
}