/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
description: |
  pending
esid: pending
---*/
// Reflect.setPrototypeOf changes an object's [[Prototype]].
var obj = {};
assert.sameValue(Object.getPrototypeOf(obj), Object.prototype);
var proto = {};
assert.sameValue(Reflect.setPrototypeOf(obj, proto), true);
assert.sameValue(Object.getPrototypeOf(obj), proto);

// It can change an object's [[Prototype]] to null.
obj = {};
assert.sameValue(Reflect.setPrototypeOf(obj, null), true);
assert.sameValue(Object.getPrototypeOf(obj), null);

// The proto argument is required too.
obj = {};
assert.throws(TypeError, () => Reflect.setPrototypeOf(obj));

// The proto argument must be either null or an object.
for (proto of [undefined, false, 0, 1.6, "that", Symbol.iterator]) {
    assert.throws(TypeError, () => Reflect.setPrototypeOf(obj, proto));
}

// Return false if the target isn't extensible.
proto = {};
obj = Object.preventExtensions(Object.create(proto));
assert.sameValue(Reflect.setPrototypeOf(obj, {}), false);
assert.sameValue(Reflect.setPrototypeOf(obj, null), false);
assert.sameValue(Reflect.setPrototypeOf(obj, proto), true);  // except if not changing anything

// Return false rather than create a [[Prototype]] cycle.
obj = {};
assert.sameValue(Reflect.setPrototypeOf(obj, obj), false);

// Don't create a [[Prototype]] cycle involving 2 objects.
obj = Object.create(proto);
assert.sameValue(Reflect.setPrototypeOf(proto, obj), false);

// Don't create a longish [[Prototype]] cycle.
for (var i = 0; i < 256; i++)
    obj = Object.create(obj);
assert.sameValue(Reflect.setPrototypeOf(proto, obj), false);

// The spec claims we should allow creating cycles involving proxies. (The
// cycle check quietly exits on encountering the proxy.)
obj = {};
var proxy = new Proxy(Object.create(obj), {});

assert.sameValue(Reflect.setPrototypeOf(obj, proxy), true);
assert.sameValue(Reflect.getPrototypeOf(obj), proxy);
assert.sameValue(Reflect.getPrototypeOf(proxy), obj);

// If a proxy handler returns a false-y value, return false.
var hits = 0;
proto = {name: "proto"};
obj = {name: "obj"};
proxy = new Proxy(obj, {
    setPrototypeOf(t, p) {
        assert.sameValue(t, obj);
        assert.sameValue(p, proto);
        hits++;
        return 0;
    }
});

assert.sameValue(Reflect.setPrototypeOf(proxy, proto), false);
assert.sameValue(hits, 1);

// For more Reflect.setPrototypeOf tests, see target.js.

