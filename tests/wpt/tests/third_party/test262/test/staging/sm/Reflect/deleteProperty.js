/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
includes: [sm/assertThrowsValue.js, deepEqual.js]
description: |
  pending
esid: pending
---*/
// Reflect.deleteProperty deletes properties.
var obj = {x: 1, y: 2};
assert.sameValue(Reflect.deleteProperty(obj, "x"), true);
assert.deepEqual(obj, {y: 2});

var arr = [1, 1, 2, 3, 5];
assert.sameValue(Reflect.deleteProperty(arr, "3"), true);
assert.deepEqual(arr, [1, 1, 2, , 5]);


// === Failure and error cases
// Since Reflect.deleteProperty is almost exactly identical to the non-strict
// `delete` operator, there is not much to test that would not be redundant.

// Returns true if no such property exists.
assert.sameValue(Reflect.deleteProperty({}, "q"), true);

// Or if it's inherited.
var proto = {x: 1};
assert.sameValue(Reflect.deleteProperty(Object.create(proto), "x"), true);
assert.sameValue(proto.x, 1);

// Return false if asked to delete a non-configurable property.
var arr = [];
assert.sameValue(Reflect.deleteProperty(arr, "length"), false);
assert.sameValue(arr.hasOwnProperty("length"), true);
assert.sameValue(Reflect.deleteProperty(this, "undefined"), false);
assert.sameValue(this.undefined, void 0);

// Return false if a Proxy's deleteProperty handler returns a false-y value.
var value;
var proxy = new Proxy({}, {
    deleteProperty(t, k) {
        return value;
    }
});
for (value of [true, false, 0, "something", {}]) {
    assert.sameValue(Reflect.deleteProperty(proxy, "q"), !!value);
}

// If a Proxy's handler method throws, the error is propagated.
proxy = new Proxy({}, {
    deleteProperty(t, k) { throw "vase"; }
});
assertThrowsValue(() => Reflect.deleteProperty(proxy, "prop"), "vase");

// Throw a TypeError if a Proxy's handler method returns true in violation of
// the object invariants.
proxy = new Proxy(Object.freeze({prop: 1}), {
    deleteProperty(t, k) { return true; }
});
assert.throws(TypeError, () => Reflect.deleteProperty(proxy, "prop"));


// === Deleting elements from `arguments`

// Non-strict arguments element becomes unmapped
function f(x, y, z) {
    assert.sameValue(Reflect.deleteProperty(arguments, "0"), true);
    arguments.x = 33;
    return x;
}
assert.sameValue(f(17, 19, 23), 17);

// Frozen non-strict arguments element
function testFrozenArguments() {
    Object.freeze(arguments);
    assert.sameValue(Reflect.deleteProperty(arguments, "0"), false);
    assert.sameValue(arguments[0], "zero");
    assert.sameValue(arguments[1], "one");
}
testFrozenArguments("zero", "one");


// For more Reflect.deleteProperty tests, see target.js and propertyKeys.js.

