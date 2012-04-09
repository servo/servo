enum handle<T> {
    _handle(@T)
}

impl methods<T> for handle<T> {
    fn get() -> @T { *self }

    fn with(f: fn(T)) {
        f(**self)
    }
}

fn handle<T:copy>(t: T) -> handle<T> {
    _handle(@t)
}
