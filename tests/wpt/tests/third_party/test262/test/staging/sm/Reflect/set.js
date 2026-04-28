/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
includes: [sm/assertThrowsValue.js, sm/non262-Reflect-shell.js, deepEqual.js]
description: |
  pending
esid: pending
---*/
// Reflect.set does property assignment.
// With three arguments, this is pretty straightforward.
var obj = {};
assert.sameValue(Reflect.set(obj, "prop", "value"), true);
assert.sameValue(obj.prop, "value");


// === Various targets

// It can assign array elements.
var arr = ["duck", "duck", "duck"];
assert.sameValue(Reflect.set(arr, 2, "goose"), true);
assert.sameValue(arr[2], "goose");

// It can extend an array.
assert.sameValue(Reflect.set(arr, 3, "Model T"), true);
assert.sameValue(arr.length, 4);

// It can truncate an array.
assert.sameValue(Reflect.set(arr, "length", 1), true);
assert.deepEqual(arr, ["duck"]);

// It won't assign to non-writable properties of String objects.
var str = new String("hello");
assert.sameValue(Reflect.set(str, "0", "y"), false);
assert.sameValue(str[0], "h");
assert.sameValue(Reflect.set(str, "length", 700), false);
assert.sameValue(str.length, 5);


// === Receivers
// The optional fourth argument is the receiver, which [[Set]] methods use for
// various things.

// On ordinary objects, if the property has a setter, the receiver is passed as
// the this-value to the setter.
var expected;
var obj = {
    set prop(v) {
        "use strict";
        assert.sameValue(v, 32);
        assert.sameValue(this, expected);
    }
};
for (expected of [obj, {}, [], 37.3]) {
    assert.sameValue(Reflect.set(obj, "prop", 32, expected), true);
}

// If the property doesn't already exist, it is defined on the receiver.
obj = {};
var obj2 = {};
assert.sameValue(Reflect.set(obj, "prop", 47, obj2), true);
assert.deepEqual(obj, {});
assert.deepEqual(Reflect.getOwnPropertyDescriptor(obj2, "prop"),
             {value: 47, writable: true, enumerable: true, configurable: true});

// If the property doesn't already exist, and the receiver isn't an object, return false.
for (var v of SOME_PRIMITIVE_VALUES) {
    assert.sameValue(Reflect.set({}, "x", 0, v), false);
}

// Receiver defaults to the target.
obj = {};
var hits;
var expectedReceiver;
var proxy = new Proxy(obj, {
    set(t, k, v, r) {
        assert.sameValue(t, obj);
        assert.sameValue(k, "key");
        assert.sameValue(v, "value");
        assert.sameValue(r, expectedReceiver); // not obj
        hits++;
        return true;
    }
});
hits = 0;
expectedReceiver = proxy;
assert.sameValue(Reflect.set(proxy, "key", "value"), true);
assert.sameValue(hits, 1);

// But not if explicitly present and undefined.
hits = 0;
expectedReceiver = undefined;
assert.sameValue(Reflect.set(proxy, "key", "value", undefined), true);
assert.sameValue(hits, 1);

// Reflect.set can be used as fallback behavior in a proxy handler .set()
// method.
var log;
obj = {
    set prop(v) {
        log += "p";
        assert.sameValue(v, "value");
        assert.sameValue(this, proxy); // not obj!
    }
};
proxy = new Proxy(obj, {
    set(t, k, v, r) {
        assert.sameValue(t, obj);
        assert.sameValue(r, proxy);
        log += "s";
        return Reflect.set(t, k, v, r);
    }
});
log = "";
assert.sameValue(Reflect.set(proxy, "prop", "value"), true);
assert.sameValue(log, "sp");


// === Cross-compartment wrapper behavior.

// When calling a cross-compartment wrapper, receiver is rewrapped for the
// target compartment.
var g = $262.createRealm().global;
if (!("assert" in g) && "assert" in globalThis)
    g.assert = assert;  // necessary when exporting to test262
if (!("assert.sameValue" in g))
    g.assert.sameValue = assert.sameValue;  // necessary in the browser
g.eval(`
     var hits;
     var obj = {
         set x(v) {
             "use strict";
             assert.sameValue(this, receiver);
             assert.sameValue(v, "xyzzy");
             hits++;
         }
     };
     var receiver = {};
`);
g.hits = 0;
assert.sameValue(Reflect.set(g.obj, "x", "xyzzy", g.receiver), true);
assert.sameValue(g.hits, 1);

// ...even when receiver is from a different compartment than target.
var receiver = {};
g.receiver = receiver;
g.hits = 0;
assert.sameValue(Reflect.set(g.obj, "x", "xyzzy", receiver), true);
assert.sameValue(g.hits, 1);

// ...even when receiver is a primtive value, even undefined.
for (receiver of SOME_PRIMITIVE_VALUES) {
    g.receiver = receiver;
    g.hits = 0;
    assert.sameValue(Reflect.set(g.obj, "x", "xyzzy", receiver), true);
    assert.sameValue(g.hits, 1);
}


// === Less than 3 arguments

// With two arguments, the value is assumed to be undefined.
obj = {};
assert.sameValue(Reflect.set(obj, "size"), true);
assert.deepEqual(Reflect.getOwnPropertyDescriptor(obj, "size"),
             {value: undefined, writable: true, enumerable: true, configurable: true});

// With just one argument, the key is "undefined".
obj = {};
assert.sameValue(Reflect.set(obj), true);
assert.deepEqual(Reflect.getOwnPropertyDescriptor(obj, "undefined"),
             {value: undefined, writable: true, enumerable: true, configurable: true});

// For the no argument-case, see target.js.


// === Failure cases

// Non-writable data property
obj = {};
Reflect.defineProperty(obj, "x", {value: 0, writable: false});
assert.sameValue(Reflect.set(obj, "x", 1), false);
assert.sameValue(obj.x, 0);

// The same, but inherited from a prototype
var obj2 = Object.create(obj);
assert.sameValue(Reflect.set(obj2, "x", 1), false);
assert.sameValue(obj2.hasOwnProperty("x"), false);
assert.sameValue(obj2.x, 0);

// Getter, no setter
obj = {};
var desc = {get: () => 12, set: undefined, enumerable: false, configurable: true};
Reflect.defineProperty(obj, "y", desc);
assert.sameValue(Reflect.set(obj, "y", 13), false);
assert.deepEqual(Reflect.getOwnPropertyDescriptor(obj, "y"), desc);

// The same, but inherited from a prototype
obj2 = Object.create(obj);
assert.sameValue(Reflect.set(obj2, "y", 1), false);
assert.sameValue(obj2.hasOwnProperty("y"), false);
assert.deepEqual(Reflect.getOwnPropertyDescriptor(obj, "y"), desc);

// Proxy set handler returns a false value
for (var no of [false, ""]) {
    var hits = 0;
    obj = {};
    var proxy = new Proxy(obj, {
        set(t, k, v, r) {
            assert.sameValue(t, obj);
            assert.sameValue(k, "x");
            assert.sameValue(v, 33);
            assert.sameValue(r, proxy);
            hits++;
            return no;
        }
    });
    assert.sameValue(Reflect.set(proxy, "x", 33), false);
    assert.sameValue(hits, 1);
    assert.sameValue("x" in obj, false);
}

// Proxy handler method throws
obj = {};
proxy = new Proxy(obj, {
    set(t, k, v, r) { throw "i don't like " + v; }
});
assertThrowsValue(() => Reflect.set(proxy, "food", "cheese"), "i don't like cheese");

// If a Proxy set handler breaks the object invariants, it's a TypeError.
for (obj of [{a: 0}, {get a() { return 0; }}]) {
    Object.freeze(obj);
    proxy = new Proxy(obj, {
        set(t, k, v, r) { return true; }
    });
    assert.throws(TypeError, () => Reflect.set(proxy, "a", "b"));
}

// Per spec, this should first call p.[[Set]]("0", 42, a) and
// then (since p has no own properties) a.[[Set]]("0", 42, a).
// The latter should not define a property on p.
var a = [0, 1, 2, 3];
var p = Object.create(a);
Reflect.set(p, "0", 42, a);
assert.sameValue(p.hasOwnProperty("0"), false);
assert.deepEqual(Reflect.getOwnPropertyDescriptor(a, "0"),
             {value: 42, writable: true, enumerable: true, configurable: true});

// Test behavior of ordinary objects' [[Set]] method (ES6 9.1.9).
// On an ordinary object, if the property key isn't present, [[Set]] calls
// receiver.[[GetOwnProperty]]() and then receiver.[[DefineProperty]]().
var log;
obj = {};
var proxyTarget = {};
var existingDescriptor, expected, defineResult;
var receiver = new Proxy(proxyTarget, {
    getOwnPropertyDescriptor(t, k) {
        log += "g";
        return existingDescriptor;
    },
    defineProperty(t, k, desc) {
        log += "d";
        assert.sameValue(t, proxyTarget);
        assert.sameValue(k, "prop");
        assert.deepEqual(desc, expected);
        return defineResult;
    }
});
existingDescriptor = undefined;
expected = {value: 5, writable: true, enumerable: true, configurable: true};
for (var defineResult of [true, false]) {
    log = "";
    assert.sameValue(Reflect.set(obj, "prop", 5, receiver), defineResult);
    assert.sameValue(log, "gd");
}

existingDescriptor = {value: 7, writable: true, enumerable: false, configurable: true};
expected = {value: 4};
for (var defineResult of [true, false]) {
    log = "";
    assert.sameValue(Reflect.set(obj, "prop", 4, receiver), defineResult);
    assert.sameValue(log, "gd");
}


// For more Reflect.set tests, see target.js and propertyKeys.js.

