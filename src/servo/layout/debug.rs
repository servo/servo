pub trait BoxedMutDebugMethods {
    fn dump(@mut self);
    fn dump_indent(@mut self, ident: uint);
    fn debug_str(@mut self) -> ~str;
}

pub trait BoxedDebugMethods {
    fn dump(@self);
    fn dump_indent(@self, ident: uint);
    fn debug_str(@self) -> ~str;
}

pub trait DebugMethods {
    fn dump(&self);
    fn dump_indent(&self, ident: uint);
    fn debug_str(&self) -> ~str;
}
