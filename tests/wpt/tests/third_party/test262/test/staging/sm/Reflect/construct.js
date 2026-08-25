/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
includes: [sm/non262-Reflect-shell.js, deepEqual.js]
description: |
  pending
esid: pending
---*/
// Reflect.construct invokes constructors.

assert.deepEqual(Reflect.construct(Object, []), {});
assert.deepEqual(Reflect.construct(String, ["hello"]), new String("hello"));

// Constructing Date creates a real Date object.
var d = Reflect.construct(Date, [1776, 6, 4]);
assert.sameValue(d instanceof Date, true);
assert.sameValue(d.getFullYear(), 1776);  // non-generic method requires real Date object

// [[Construct]] methods don't necessarily create new objects.
var obj = {};
assert.sameValue(Reflect.construct(Object, [obj]), obj);


// === Various kinds of constructors
// We've already seen some builtin constructors.
//
// JS functions:
function f(x) { this.x = x; }
assert.deepEqual(Reflect.construct(f, [3]), new f(3));
f.prototype = Array.prototype;
assert.deepEqual(Reflect.construct(f, [3]), new f(3));

// Bound functions:
var bound = f.bind(null, "carrot");
assert.deepEqual(Reflect.construct(bound, []), new bound);

// Classes:
class Base {
    constructor(...args) {
        this.args = args;
        this.newTarget = new.target;
    }
}
class Derived extends Base {
    constructor(...args) { super(...args); }
}

assert.deepEqual(Reflect.construct(Base, []), new Base);
assert.deepEqual(Reflect.construct(Derived, [7]), new Derived(7));
g = Derived.bind(null, "q");
assert.deepEqual(Reflect.construct(g, [8, 9]), new g(8, 9));

// Cross-compartment wrappers:
var g = $262.createRealm().global;
var local = {here: this};
g.eval("function F(arg) { this.arg = arg }");
assert.deepEqual(Reflect.construct(g.F, [local]), new g.F(local));

// If first argument to Reflect.construct isn't a constructor, it throws a
// TypeError.
var nonConstructors = [
    {},
    Reflect.construct,  // builtin functions aren't constructors
    x => x + 1,
    Math.max.bind(null, 0),  // bound non-constructors aren't constructors
    ((x, y) => x > y).bind(null, 0),

    // A Proxy to a non-constructor function isn't a constructor, even if a
    // construct handler is present.
    new Proxy(Reflect.construct, {construct(){}}),
];
for (var obj of nonConstructors) {
    assert.throws(TypeError, () => Reflect.construct(obj, []));
    assert.throws(TypeError, () => Reflect.construct(obj, [], Object));
}


// === new.target tests

// If the newTarget argument to Reflect.construct is missing, the target is used.
function checkNewTarget() {
    assert.sameValue(new.target, expected);
    expected = undefined;
}
var expected = checkNewTarget;
Reflect.construct(checkNewTarget, []);

// The newTarget argument is correctly passed to the constructor.
var constructors = [Object, Function, f, bound];
for (var ctor of constructors) {
    expected = ctor;
    Reflect.construct(checkNewTarget, [], ctor);
    assert.sameValue(expected, undefined);
}

// The newTarget argument must be a constructor.
for (var v of SOME_PRIMITIVE_VALUES.concat(nonConstructors)) {
    assert.throws(TypeError, () => Reflect.construct(checkNewTarget, [], v));
}

// The builtin Array constructor uses new.target.prototype and always
// creates a real array object.
function someConstructor() {}
var result = Reflect.construct(Array, [], someConstructor);
assert.sameValue(Reflect.getPrototypeOf(result), someConstructor.prototype);
assert.sameValue(result.length, 0);
assert.sameValue(Array.isArray(result), true);


// For more Reflect.construct tests, see target.js and argumentsList.js.

