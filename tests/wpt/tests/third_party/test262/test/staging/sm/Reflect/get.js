/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
description: |
  pending
esid: pending
---*/
// Reflect.get gets the value of a property.

var x = {p: 1};
assert.sameValue(Reflect.get(x, "p"), 1);
assert.sameValue(Reflect.get(x, "toString"), Object.prototype.toString);
assert.sameValue(Reflect.get(x, "missing"), undefined);


// === Various targets

// Array
assert.sameValue(Reflect.get([], 700), undefined);
assert.sameValue(Reflect.get(["zero", "one"], 1), "one");

// TypedArray
assert.sameValue(Reflect.get(new Uint8Array([0, 1, 2, 3, 4, 5, 6, 7]), 7), 7);

// Treatment of NaN
var f = new Float64Array([NaN]);
var u = new Uint32Array(f.buffer);
u[0]++;
u[1]++;
assert.sameValue(f[0], NaN);
assert.sameValue(Reflect.get(f, 0), NaN);

// Proxy with no get handler
assert.sameValue(Reflect.get(new Proxy(x, {}), "p"), 1);

// Proxy with a get handler
var obj = new Proxy(x, {
    get(t, k, r) { return k + "ful"; }
});
assert.sameValue(Reflect.get(obj, "mood"), "moodful");

// Exceptions thrown by a proxy's get handler are propagated.
assert.throws(TypeError, () => Reflect.get(obj, Symbol()));

// Ordinary object, property has a setter and no getter
obj = {set name(x) {}};
assert.sameValue(Reflect.get(obj, "name"), undefined);


// === Receiver

// Receiver argument is passed to getters as the this-value.
obj = { get x() { return this; }};
assert.sameValue(Reflect.get(obj, "x", Math), Math);
assert.sameValue(Reflect.get(Object.create(obj), "x", JSON), JSON);

// If missing, target is passed instead.
assert.sameValue(Reflect.get(obj, "x"), obj);

// Receiver argument is passed to the proxy get handler.
obj = new Proxy({}, {
    get(t, k, r) {
        assert.sameValue(k, "itself");
        return r;
    }
});
assert.sameValue(Reflect.get(obj, "itself"), obj);
assert.sameValue(Reflect.get(obj, "itself", Math), Math);
assert.sameValue(Reflect.get(Object.create(obj), "itself", Math), Math);

// The receiver shouldn't have to be an object
assert.sameValue(Reflect.get(obj, "itself", 37.2), 37.2);

// For more Reflect.get tests, see target.js and propertyKeys.js.

