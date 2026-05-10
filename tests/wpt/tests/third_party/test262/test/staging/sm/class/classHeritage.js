// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: |
  pending
esid: pending
---*/
// It's an error to have a non-constructor as your heritage
assert.throws(TypeError, () => eval(`class a extends Math.sin {
                                        constructor() { }
                                    }`));
assert.throws(TypeError, () => eval(`(class a extends Math.sin {
                                        constructor() { }
                                    })`));

// Unless it's null, in which case it works like a normal class, except that
// the prototype object does not inherit from Object.prototype.
class basic {
    constructor() { }
}
class nullExtends extends null {
    constructor() { }
}
var nullExtendsExpr = class extends null { constructor() { } };

assert.sameValue(Object.getPrototypeOf(basic), Function.prototype);
assert.sameValue(Object.getPrototypeOf(basic.prototype), Object.prototype);

for (let extension of [nullExtends, nullExtendsExpr]) {
    assert.sameValue(Object.getPrototypeOf(extension), Function.prototype);
    assert.sameValue(Object.getPrototypeOf(extension.prototype), null);
}

var baseMethodCalled;
var staticMethodCalled;
var overrideCalled;
class base {
    constructor() { };
    method() { baseMethodCalled = true; }
    static staticMethod() { staticMethodCalled = true; }
    override() { overrideCalled = "base" }
}
class derived extends base {
    constructor() { super(); };
    override() { overrideCalled = "derived"; }
}
var derivedExpr = class extends base {
    constructor() { super(); };
    override() { overrideCalled = "derived"; }
};

// Make sure we get the right object layouts.
for (let extension of [derived, derivedExpr]) {
    baseMethodCalled = false;
    staticMethodCalled = false;
    overrideCalled = "";
    // Make sure we get the right object layouts.
    assert.sameValue(Object.getPrototypeOf(extension), base);
    assert.sameValue(Object.getPrototypeOf(extension.prototype), base.prototype);

    // We do inherit the methods, right?
    (new extension()).method();
    assert.sameValue(baseMethodCalled, true);

    // But we can still override them?
    (new extension()).override();
    assert.sameValue(overrideCalled, "derived");

    // What about the statics?
    extension.staticMethod();
    assert.sameValue(staticMethodCalled, true);
}

// Gotta extend an object, or null.
function nope() {
    class Foo extends "Bar" {
        constructor() { }
    }
}
function nopeExpr() {
    (class extends "Bar" {
        constructor() { }
     });
}
assert.throws(TypeError, nope);
assert.throws(TypeError, nopeExpr);

// The .prototype of the extension must be an object, or null.
nope.prototype = "not really, no";
function stillNo() {
    class Foo extends nope {
        constructor() { }
    }
}
function stillNoExpr() {
    (class extends nope {
        constructor() { }
     });
}
assert.throws(TypeError, stillNo);
assert.throws(TypeError, stillNoExpr);

