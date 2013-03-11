pub trait BoxedMutDebugMethods {
    pure fn dump(@mut self);
    pure fn dump_indent(@mut self, ident: uint);
    pure fn debug_str(@mut self) -> ~str;
}

pub trait BoxedDebugMethods {
    pure fn dump(@self);
    pure fn dump_indent(@self, ident: uint);
    pure fn debug_str(@self) -> ~str;
}

pub trait DebugMethods {
    pure fn dump(&self);
    pure fn dump_indent(&self, ident: uint);
    pure fn debug_str(&self) -> ~str;
}
