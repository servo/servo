import task::*;
import comm::*;

enum msg {
    do_layout,
    exit
}

fn layout(lister: chan<lister::msg>) -> chan<msg> {
    spawn_listener::<msg> {|po|
        loop {
            alt recv(po) {
              do_layout {
                send(lister, lister::build)
              }
              exit {
                break;
              }
            }
        }
    }
}
