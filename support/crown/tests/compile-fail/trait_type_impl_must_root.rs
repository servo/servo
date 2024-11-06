#[crown::unrooted_must_root_lint::must_root]
struct Foo;

trait Trait {
    type F;
    //~^ ERROR: Type trait declaration must be marked with #[crown::unrooted_must_root_lint::must_root] to allow binding must_root types in associate types
}

struct TypeHolder;

impl Trait for TypeHolder {
    // type F in trait must be also marked as must_root if we want to do this
    type F = Foo;
}

fn main() {}