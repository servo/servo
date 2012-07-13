// Timing functions.
import std::time::precise_time_ns;

fn time(msg: str, callback: fn()) {
    let start_time = precise_time_ns();
    callback();
    let end_time = precise_time_ns();
    #debug("%s took %u ms", msg, ((end_time - start_time) / 1000000u64) as uint);
}


