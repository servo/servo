import libc::c_char;

type name_pool = @{
    mut strbufs: [str]
};

fn name_pool() -> name_pool {
    @{mut strbufs: []}
}

impl methods for name_pool {
    fn add(-s: str) -> *c_char {
        let c_str = str::as_c_str(s) { |bytes| bytes };
        self.strbufs += [s]; // in theory, this should *move* the str in here..
        ret c_str; // ...and so this ptr ought to be valid.
    }
}