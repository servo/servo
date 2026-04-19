/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
includes: [deepEqual.js]
flags:
  - noStrict
description: |
  pending
esid: pending
---*/
// Basic tests for standard Object APIs interacting with symbols.

// Object.defineProperty
function F() {}
var f = new F;
Object.defineProperty(f, Symbol.for("name"), {
    configurable: true,
    value: "eff"
});
assert.sameValue("name" in f, false);
assert.sameValue("Symbol(name)" in f, false);
assert.sameValue(Symbol.for("name") in f, true);
assert.sameValue(f[Symbol.for("name")], "eff");

// Object.defineProperties
function D() {}
var descs = new D;
var s1 = Symbol("s1");
var hits = 0;
descs[s1] = {
    get: () => hits++,
    set: undefined,
    enumerable: true,
    configurable: true
};
var s2 = Symbol("s2");
descs[s2] = {
    value: {},
    writable: true,
    enumerable: false,
    configurable: true
};
var s3 = Symbol("s3");
D.prototype[s3] = {value: "FAIL"};
assert.sameValue(Object.defineProperties(f, descs), f);
assert.sameValue(s1 in f, true);
assert.sameValue(f[s1], 0);
assert.sameValue(hits, 1);
assert.sameValue(s2 in f, true);
assert.sameValue(f[s2], descs[s2].value);
assert.sameValue(s3 in f, false);

// Object.create
var n = Object.create({}, descs);
assert.sameValue(s1 in n, true);
assert.sameValue(n[s1], 1);
assert.sameValue(hits, 2);
assert.sameValue(s2 in n, true);
assert.sameValue(n[s2], descs[s2].value);
assert.sameValue(s3 in n, false);

// Object.getOwnPropertyDescriptor
var desc = Object.getOwnPropertyDescriptor(n, s1);
assert.deepEqual(desc, descs[s1]);
assert.sameValue(desc.get, descs[s1].get);
desc = Object.getOwnPropertyDescriptor(n, s2);
assert.deepEqual(desc, descs[s2]);
assert.sameValue(desc.value, descs[s2].value);

// Object.prototype.hasOwnProperty
assert.sameValue(descs.hasOwnProperty(s1), true);
assert.sameValue(descs.hasOwnProperty(s2), true);
assert.sameValue(descs.hasOwnProperty(s3), false);
assert.sameValue([].hasOwnProperty(Symbol.iterator), false);
assert.sameValue(Array.prototype.hasOwnProperty(Symbol.iterator), true);

// Object.prototype.propertyIsEnumerable
assert.sameValue(n.propertyIsEnumerable(s1), true);
assert.sameValue(n.propertyIsEnumerable(s2), false);
assert.sameValue(n.propertyIsEnumerable(s3), false);  // no such property
assert.sameValue(D.prototype.propertyIsEnumerable(s3), true);
assert.sameValue(descs.propertyIsEnumerable(s3), false); // inherited properties are not considered

// Object.preventExtensions
var obj = {};
obj[s1] = 1;
assert.sameValue(Object.preventExtensions(obj), obj);
assert.throws(TypeError, function () { "use strict"; obj[s2] = 2; });
obj[s2] = 2;  // still no effect
assert.sameValue(s2 in obj, false);

// Object.isSealed, Object.isFrozen
assert.sameValue(Object.isSealed(obj), false);
assert.sameValue(Object.isFrozen(obj), false);
assert.sameValue(delete obj[s1], true);
assert.sameValue(Object.isSealed(obj), true);
assert.sameValue(Object.isFrozen(obj), true);

obj = {};
obj[s1] = 1;
Object.preventExtensions(obj);
Object.defineProperty(obj, s1, {configurable: false});  // still writable
assert.sameValue(Object.isSealed(obj), true);
assert.sameValue(Object.isFrozen(obj), false);
obj[s1] = 2;
assert.sameValue(obj[s1], 2);
Object.defineProperty(obj, s1, {writable: false});
assert.sameValue(Object.isFrozen(obj), true);

// Object.seal, Object.freeze
var obj = {};
obj[s1] = 1;
Object.seal(obj);
desc = Object.getOwnPropertyDescriptor(obj, s1);
assert.sameValue(desc.configurable, false);
assert.sameValue(desc.writable, true);
Object.freeze(obj);
assert.sameValue(Object.getOwnPropertyDescriptor(obj, s1).writable, false);

// Object.setPrototypeOf purges caches for symbol-keyed properties.
var proto = {};
proto[s1] = 1;
Object.defineProperty(proto, s2, {
    get: () => 2,
    set: v => undefined
});
var obj = Object.create(proto);
var last1, last2;
var N = 9;
for (var i = 0; i < N; i++) {
    last1 = obj[s1];
    last2 = obj[s2];
    obj[s2] = "marker";
    if (i === N - 2)
        Object.setPrototypeOf(obj, {});
}
assert.sameValue(last1, undefined);
assert.sameValue(last2, undefined);
assert.sameValue(obj.hasOwnProperty(s2), true);
assert.sameValue(obj[s2], "marker");

