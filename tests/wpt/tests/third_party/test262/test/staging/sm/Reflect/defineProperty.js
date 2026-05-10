/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
features:
  - IsHTMLDDA
includes: [sm/assertThrowsValue.js, sm/non262-Reflect-shell.js, deepEqual.js]
description: |
  pending
esid: pending
---*/
// Reflect.defineProperty defines properties.
var obj = {};
assert.sameValue(Reflect.defineProperty(obj, "x", {value: 7}), true);
assert.sameValue(obj.x, 7);
var desc = Reflect.getOwnPropertyDescriptor(obj, "x");
assert.deepEqual(desc, {value: 7,
                    writable: false,
                    enumerable: false,
                    configurable: false});

// Reflect.defineProperty can define a symbol-keyed property.
var key = Symbol(":o)");
assert.sameValue(Reflect.defineProperty(obj, key, {value: 8}), true);
assert.sameValue(obj[key], 8);

// array .length property
obj = [1, 2, 3, 4, 5];
assert.sameValue(Reflect.defineProperty(obj, "length", {value: 4}), true);
assert.deepEqual(obj, [1, 2, 3, 4]);

// The target can be a proxy.
obj = {};
var proxy = new Proxy(obj, {
    defineProperty(t, id, desc) {
        t[id] = 1;
        return true;
    }
});
assert.sameValue(Reflect.defineProperty(proxy, "prop", {value: 7}), true);
assert.sameValue(obj.prop, 1);
assert.sameValue(delete obj.prop, true);
assert.sameValue("prop" in obj, false);

// The attributes object is re-parsed, not passed through to the
// handler.defineProperty method.
obj = {};
var attributes = {
    configurable: 17,
    enumerable: undefined,
    value: null
};
proxy = new Proxy(obj, {
    defineProperty(t, id, desc) {
        assert.sameValue(desc !== attributes, true);
        assert.sameValue(desc.configurable, true);
        assert.sameValue(desc.enumerable, false);
        assert.sameValue(desc.value, null);
        assert.sameValue("writable" in desc, false);
        return 15;  // and the return value here is coerced to boolean
    }
});
assert.sameValue(Reflect.defineProperty(proxy, "prop", attributes), true);


// === Failure and error cases
//
// Reflect.defineProperty behaves much like Object.defineProperty, which has
// extremely thorough tests elsewhere, and the implementation is largely
// shared. Duplicating those tests with Reflect.defineProperty would be a
// big waste.
//
// However, certain failures cause Reflect.defineProperty to return false
// without throwing a TypeError (unlike Object.defineProperty). So here we test
// many error cases to check that behavior.

// missing attributes argument
assert.throws(TypeError, () => Reflect.defineProperty(obj, "y"));

// non-object attributes argument
for (var attributes of SOME_PRIMITIVE_VALUES) {
    assert.throws(TypeError, () => Reflect.defineProperty(obj, "y", attributes));
}

// inextensible object
obj = Object.preventExtensions({});
assert.sameValue(Reflect.defineProperty(obj, "prop", {value: 4}), false);

// inextensible object with irrelevant inherited property
obj = Object.preventExtensions(Object.create({"prop": 3}));
assert.sameValue(Reflect.defineProperty(obj, "prop", {value: 4}), false);

// redefine nonconfigurable to configurable
obj = Object.freeze({prop: 1});
assert.sameValue(Reflect.defineProperty(obj, "prop", {configurable: true}), false);

// redefine enumerability of nonconfigurable property
obj = Object.freeze(Object.defineProperties({}, {
    x: {enumerable: true,  configurable: false, value: 0},
    y: {enumerable: false, configurable: false, value: 0},
}));
assert.sameValue(Reflect.defineProperty(obj, "x", {enumerable: false}), false);
assert.sameValue(Reflect.defineProperty(obj, "y", {enumerable: true}), false);

// redefine nonconfigurable data to accessor property, or vice versa
obj = Object.seal({x: 1, get y() { return 2; }});
assert.sameValue(Reflect.defineProperty(obj, "x", {get() { return 2; }}), false);
assert.sameValue(Reflect.defineProperty(obj, "y", {value: 1}), false);

// redefine nonwritable, nonconfigurable property as writable
obj = Object.freeze({prop: 0});
assert.sameValue(Reflect.defineProperty(obj, "prop", {writable: true}), false);
assert.sameValue(Reflect.defineProperty(obj, "prop", {writable: false}), true);  // no-op

// change value of nonconfigurable nonwritable property
obj = Object.freeze({prop: 0});
assert.sameValue(Reflect.defineProperty(obj, "prop", {value: -0}), false);
assert.sameValue(Reflect.defineProperty(obj, "prop", {value: +0}), true);  // no-op

// change getter or setter
function g() {}
function s(x) {}
obj = {};
Object.defineProperty(obj, "prop", {get: g, set: s, configurable: false});
assert.sameValue(Reflect.defineProperty(obj, "prop", {get: s}), false);
assert.sameValue(Reflect.defineProperty(obj, "prop", {get: g}), true);  // no-op
assert.sameValue(Reflect.defineProperty(obj, "prop", {set: g}), false);
assert.sameValue(Reflect.defineProperty(obj, "prop", {set: s}), true);  // no-op

// Proxy defineProperty handler method that returns false
var falseValues = [false, 0, -0, "", NaN, null, undefined, $262.IsHTMLDDA];
var value;
proxy = new Proxy({}, {
    defineProperty(t, id, desc) {
        return value;
    }
});
for (value of falseValues) {
    assert.sameValue(Reflect.defineProperty(proxy, "prop", {value: 1}), false);
}

// Proxy defineProperty handler method returns true, in violation of invariants.
// Per spec, this is a TypeError, not a false return.
obj = Object.freeze({x: 1});
proxy = new Proxy(obj, {
    defineProperty(t, id, desc) {
        return true;
    }
});
assert.throws(TypeError, () => Reflect.defineProperty(proxy, "x", {value: 2}));
assert.throws(TypeError, () => Reflect.defineProperty(proxy, "y", {value: 0}));
assert.sameValue(Reflect.defineProperty(proxy, "x", {value: 1}), true);

// The second argument is converted ToPropertyKey before any internal methods
// are called on the first argument.
var poison =
  (counter => new Proxy({}, new Proxy({}, { get() { throw counter++; } })))(42);
assertThrowsValue(() => {
    Reflect.defineProperty(poison, {
        toString() { throw 17; },
        valueOf() { throw 8675309; }
    }, poison);
}, 17);


// For more Reflect.defineProperty tests, see target.js and propertyKeys.js.

