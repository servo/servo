// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: |
  pending
esid: pending
---*/
// It is possible to override Function.prototype[@@hasInstance].
let passed = false;
let obj = { foo: true };
let C = function(){};

Object.defineProperty(C, Symbol.hasInstance, {
  value: function(inst) { passed = inst.foo; return false; }
});

assert.sameValue(obj instanceof C, false);
assert.sameValue(passed, true);

{
    let obj = {
        [Symbol.hasInstance](v) { return true; },
    };
    let whatevs = {};
    assert.sameValue(whatevs instanceof obj, true);
}

{

    function zzzz() {};
    let xxxx = new zzzz();
    assert.sameValue(xxxx instanceof zzzz, true);
    assert.sameValue(zzzz[Symbol.hasInstance](xxxx), true);

}

// Non-callable objects should return false.
const nonCallables = [
    1,
    undefined,
    null,
    "nope",
]

for (let nonCallable of nonCallables) {
    assert.sameValue(nonCallable instanceof Function, false);
    assert.sameValue(nonCallable instanceof Object, false);
}

// Non-callables should throw when used on the right hand side
// of `instanceof`.
assert.throws(TypeError, () => {
    function foo() {};
    let obj = {};
    foo instanceof obj;
});

// Non-callables do not throw for overridden methods
let o = {[Symbol.hasInstance](v) { return true; }}
assert.sameValue(1 instanceof o, true);

// Non-callables return false instead of an exception when
// Function.prototype[Symbol.hasInstance] is called directly.
for (let nonCallable of nonCallables) {
    assert.sameValue(Function.prototype[Symbol.hasInstance].call(nonCallable, Object), false);
}

// It should be possible to call the Symbol.hasInstance method directly.
assert.sameValue(Function.prototype[Symbol.hasInstance].call(Function, () => 1), true);
assert.sameValue(Function.prototype[Symbol.hasInstance].call(Function, Object), true);
assert.sameValue(Function.prototype[Symbol.hasInstance].call(Function, null), false);
assert.sameValue(Function.prototype[Symbol.hasInstance].call(Function, Array), true);
assert.sameValue(Function.prototype[Symbol.hasInstance].call(Object, Array), true);
assert.sameValue(Function.prototype[Symbol.hasInstance].call(Array, Function), false);
assert.sameValue(Function.prototype[Symbol.hasInstance].call(({}), Function), false);
assert.sameValue(Function.prototype[Symbol.hasInstance].call(), false)
assert.sameValue(Function.prototype[Symbol.hasInstance].call(({})), false)

// Ensure that bound functions are unwrapped properly
let bindme = {x: function() {}};
let instance = new bindme.x();
let xOuter = bindme.x;
let bound = xOuter.bind(bindme);
let doubleBound = bound.bind(bindme);
let tripleBound = bound.bind(doubleBound);
assert.sameValue(Function.prototype[Symbol.hasInstance].call(bound, instance), true);
assert.sameValue(Function.prototype[Symbol.hasInstance].call(doubleBound, instance), true);
assert.sameValue(Function.prototype[Symbol.hasInstance].call(tripleBound, instance), true);

// Function.prototype[Symbol.hasInstance] is not configurable
let desc = Object.getOwnPropertyDescriptor(Function.prototype, Symbol.hasInstance);
assert.sameValue(desc.configurable, false);

// Attempting to use a non-callable @@hasInstance triggers a type error
// Bug 1280892
assert.throws(TypeError, () => {
    var fun = function() {}
    var p = new Proxy(fun, {
        get(target, key) {
            return /not-callable/;
        }
    });
    fun instanceof p;
});


