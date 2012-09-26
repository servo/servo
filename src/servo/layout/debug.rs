trait BoxedDebugMethods {
    fn dump(@self);
    fn dump_indent(@self, ident: uint);
    fn debug_str(@self) -> ~str;
}

trait DebugMethods {
    fn dump(&self);
    fn dump_indent(&self, ident: uint);
    fn debug_str(&self) -> ~str;
}
