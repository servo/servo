/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
includes: [compareArray.js]
description: |
  pending
esid: pending
---*/
// Reflect.ownKeys(obj) returns an array of an object's own property keys.

// Test that Reflect.ownKeys gets the expected result when applied to various
// objects. (These tests also check the basics: that the result is an array,
// that its prototype is correct, etc.)
var sym = Symbol.for("comet");
var sym2 = Symbol.for("meteor");
var cases = [
    {object: {z: 3, y: 2, x: 1},
     keys: ["z", "y", "x"]},
    {object: [],
     keys: ["length"]},
    {object: new Int8Array(4),
     keys: ["0", "1", "2", "3"]},
    {object: new Proxy({a: 7}, {}),
     keys: ["a"]},
    {object: {[sym]: "ok"},
     keys: [sym]},
    {object: {[sym]: 0,  // test 9.1.12 ordering
              "str": 0,
              "773": 0,
              "0": 0,
              [sym2]: 0,
              "-1": 0,
              "8": 0,
              "second str": 0},
     keys: ["0", "8", "773",  // indexes in numeric order
            "str", "-1", "second str", // strings in insertion order
            sym, sym2]}, // symbols in insertion order
    {object: $262.createRealm().global.Math,  // cross-compartment wrapper
     keys: Reflect.ownKeys(Math)}
];
for (var {object, keys} of cases)
    assert.compareArray(Reflect.ownKeys(object), keys);

// Reflect.ownKeys() creates a new array each time it is called.
var object = {}, keys = [];
for (var i = 0; i < 3; i++) {
    var newKeys = Reflect.ownKeys(object);
    assert.sameValue(newKeys !== keys, true);
    keys = newKeys;
}

// Proxy behavior with successful ownKeys() handler
keys = ["str", "0"];
obj = {};
proxy = new Proxy(obj, {
    ownKeys() { return keys; }
});
var actual = Reflect.ownKeys(proxy);
assert.compareArray(actual, keys);  // we get correct answers
assert.sameValue(actual !== keys, true);  // but not the same object

// If a proxy breaks invariants, a TypeError is thrown.
var obj = Object.preventExtensions({});
var proxy = new Proxy(obj, {
    ownKeys() { return ["something"]; }
});
assert.throws(TypeError, () => Reflect.ownKeys(proxy));

// For more Reflect.ownKeys tests, see target.js.

