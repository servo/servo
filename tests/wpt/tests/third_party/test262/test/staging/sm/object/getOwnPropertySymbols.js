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

assert.compareArray(Object.getOwnPropertySymbols({}), []);

// String keys are ignored.
assert.sameValue(Object.getOwnPropertySymbols({a: 1, b: 2}).length, 0);
assert.sameValue(Object.getOwnPropertySymbols([0, 1, 2, 3]).length, 0);

// Symbol keys are observed.
var iterable = {};
Object.defineProperty(iterable, Symbol.iterator, {
    value: () => [][Symbol.iterator]()
});
assert.compareArray(Object.getOwnPropertySymbols(iterable), [Symbol.iterator]);
assert.compareArray(Object.getOwnPropertySymbols(new Proxy(iterable, {})), [Symbol.iterator]);

// Test on an object with a thousand own properties.
var obj = {};
for (var i = 0; i < 1000; i++) {
    obj[Symbol.for("x" + i)] = 1;
}
assert.sameValue(Object.getOwnPropertyNames(obj).length, 0);
var symbols = Object.getOwnPropertySymbols(obj);
assert.sameValue(symbols.length, 1000);
assert.sameValue(symbols.indexOf(Symbol.for("x0")) !== -1, true);
assert.sameValue(symbols.indexOf(Symbol.for("x241")) !== -1, true);
assert.sameValue(symbols.indexOf(Symbol.for("x999")) !== -1, true);
assert.sameValue(Object.getOwnPropertySymbols(new Proxy(obj, {})).length, 1000);

// The prototype chain is not consulted.
assert.sameValue(Object.getOwnPropertySymbols(Object.create(obj)).length, 0);
assert.sameValue(Object.getOwnPropertySymbols(new Proxy(Object.create(obj), {})).length, 0);

// Primitives are coerced to objects; but there are never any symbol-keyed
// properties on the resulting wrapper objects.
assert.throws(TypeError, () => Object.getOwnPropertySymbols());
assert.throws(TypeError, () => Object.getOwnPropertySymbols(undefined));
assert.throws(TypeError, () => Object.getOwnPropertySymbols(null));
for (var primitive of [true, 1, 3.14, "hello", Symbol()])
    assert.sameValue(Object.getOwnPropertySymbols(primitive).length, 0);

assert.sameValue(Object.getOwnPropertySymbols.length, 1);
