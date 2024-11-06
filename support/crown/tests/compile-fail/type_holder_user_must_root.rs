struct Foo(i32);

struct Bar<TH: TypeHolderTrait>(TH::F);
//~^ ERROR: Type must be rooted, use #[crown::unrooted_must_root_lint::must_root] on the struct definition to propagate

trait TypeHolderTrait {
    #[crown::unrooted_must_root_lint::must_root]
    type F;
}

struct TypeHolder;

impl TypeHolderTrait for TypeHolder {
    type F = Foo;
}

fn main() {}
