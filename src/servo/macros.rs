{
    macro_rules! move_ref {
        { $x:expr } => { unsafe { let y <- *ptr::addr_of(*$x); y } }
    }

    macro_rules! move_val {
        { $x:expr } => { unsafe { let y <- *ptr::addr_of(*$x); y } }
    }
}
