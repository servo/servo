enum handle<T> = @T;

impl methods<T> for handle<T> {
    fn get() -> @T { *self }
}

