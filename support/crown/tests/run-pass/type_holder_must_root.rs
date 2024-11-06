#[crown::unrooted_must_root_lint::must_root]
struct Foo(i32);
#[crown::unrooted_must_root_lint::must_root]
struct Bar<TH: TypeHolderTrait>(TH::F, TH::B);

struct Baz(i32);

trait TypeHolderTrait {
    #[crown::unrooted_must_root_lint::must_root]
    type F;
    type B;
}

struct TypeHolder;

impl TypeHolderTrait for TypeHolder {
    type F = Foo;
    type B = Baz;
}

fn main() {}