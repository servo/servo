// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: |
  pending
esid: pending
---*/
// Named class definitions should create an immutable inner binding.
// Since all code in classes is in strict mode, attempts to mutate it
// should throw.
class Foof { constructor() { }; tryBreak() { Foof = 4; } }
for (let result of [Foof, class Bar { constructor() { }; tryBreak() { Bar = 4; } }])
    assert.throws(TypeError, () => new result().tryBreak());

{
    class foo { constructor() { }; tryBreak() { foo = 4; } }
    for (let result of [foo, class Bar { constructor() { }; tryBreak() { Bar = 4 } }])
        assert.throws(TypeError, () => new result().tryBreak());
}

// TDZ applies to inner bindings
assert.throws(ReferenceError, ()=>eval(`class Bar {
                                    constructor() { };
                                    [Bar] () { };
                                 }`));

assert.throws(ReferenceError, ()=>eval(`(class Bar {
                                    constructor() { };
                                    [Bar] () { };
                                 })`));

// There's no magic "inner binding" global
{
    class Foo {
        constructor() { };
        test() {
            class Bar {
                constructor() { }
                test() { return Foo === Bar }
            }
            return new Bar().test();
        }
    }
    assert.sameValue(new Foo().test(), false);
    assert.sameValue(new class foo {
        constructor() { };
        test() {
            return new class bar {
                constructor() { }
                test() { return foo === bar }
            }().test();
        }
    }().test(), false);
}

// Inner bindings are shadowable
{
    class Foo {
        constructor() { }
        test(Foo) { return Foo; }
    }
    assert.sameValue(new Foo().test(4), 4);
    assert.sameValue(new class foo {
        constructor() { };
        test(foo) { return foo }
    }().test(4), 4);
}

// Inner bindings in expressions should shadow even existing names.
class Foo { constructor() { } static method() { throw new Error("NO!"); } }
assert.sameValue(new class Foo {
            constructor() { };
            static method() { return 4; };
            test() { return Foo.method(); }
         }().test(), 4);

// The outer binding is distinct from the inner one
{
    let orig_X;

    class X {
        constructor() { }
        f() { assert.sameValue(X, orig_X); }
    }

    orig_X = X;
    X = 13;
    assert.sameValue(X, 13);
    new orig_X().f();
}

