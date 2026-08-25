// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
includes: [compareArray.js]
flags:
  - noStrict
description: |
  pending
esid: pending
---*/

// ES6 7.1.1 ToPrimitive(input [, PreferredType]) specifies a new extension
// point in the language. Objects can override the behavior of ToPrimitive
// somewhat by supporting the method obj[@@toPrimitive](hint).
//
// (Rationale: ES5 had a [[DefaultValue]] internal method, overridden only by
// Date objects. The change in ES6 is to make [[DefaultValue]] a plain old
// method. This allowed ES6 to eliminate the [[DefaultValue]] internal method,
// simplifying the meta-object protocol and thus proxies.)

// obj[Symbol.toPrimitive]() is called whenever the ToPrimitive algorithm is invoked.
var expectedThis, expectedHint;
var obj = {
    [Symbol.toPrimitive](hint, ...rest) {
        assert.sameValue(this, expectedThis);
        assert.sameValue(hint, expectedHint);
        assert.sameValue(rest.length, 0);
        return 2015;
    }
};
expectedThis = obj;
expectedHint = "string";
assert.sameValue(String(obj), "2015");
expectedHint = "number";
assert.sameValue(Number(obj), 2015);

// It is called even through proxies.
var proxy = new Proxy(obj, {});
expectedThis = proxy;
expectedHint = "default";
assert.sameValue("ES" + proxy, "ES2015");

// It is called even through additional proxies and the prototype chain.
proxy = new Proxy(Object.create(proxy), {});
expectedThis = proxy;
expectedHint = "default";
assert.sameValue("ES" + (proxy + 1), "ES2016");

// It is not called if the operand is already a primitive.
var ok = true;
for (var constructor of [Boolean, Number, String, Symbol]) {
    constructor.prototype[Symbol.toPrimitive] = function () {
        ok = false;
        throw "FAIL";
    };
}
assert.sameValue(Number(true), 1);
assert.sameValue(Number(77.7), 77.7);
assert.sameValue(Number("123"), 123);
assert.throws(TypeError, () => Number(Symbol.iterator));
assert.sameValue(String(true), "true");
assert.sameValue(String(77.7), "77.7");
assert.sameValue(String("123"), "123");
assert.sameValue(String(Symbol.iterator), "Symbol(Symbol.iterator)");
assert.sameValue(ok, true);

// Converting a primitive symbol to another primitive type throws even if you
// delete the @@toPrimitive method from Symbol.prototype.
delete Symbol.prototype[Symbol.toPrimitive];
var sym = Symbol("ok");
assert.throws(TypeError, () => `${sym}`);
assert.throws(TypeError, () => Number(sym));
assert.throws(TypeError, () => "" + sym);

// However, having deleted that method, converting a Symbol wrapper object does
// work: it calls Symbol.prototype.toString().
obj = Object(sym);
assert.sameValue(String(obj), "Symbol(ok)");
assert.sameValue(`${obj}`, "Symbol(ok)");

// Deleting valueOf as well makes numeric conversion also call toString().
delete Symbol.prototype.valueOf;
delete Object.prototype.valueOf;
assert.sameValue(Number(obj), NaN);
Symbol.prototype.toString = function () { return "2060"; };
assert.sameValue(Number(obj), 2060);

// Deleting Date.prototype[Symbol.toPrimitive] changes the result of addition
// involving Date objects.
var d = new Date;
assert.sameValue(0 + d, 0 + d.toString());
delete Date.prototype[Symbol.toPrimitive];
assert.sameValue(0 + d, 0 + d.valueOf());

// If @@toPrimitive, .toString, and .valueOf are all missing, we get a
// particular sequence of property accesses, followed by a TypeError exception.
var log = [];
function doGet(target, propertyName, receiver) {
    log.push(propertyName);
}
var handler = new Proxy({}, {
    get(target, trapName, receiver) {
        if (trapName !== "get")
            throw `FAIL: system tried to access handler method: ${String(trapName)}`;
        return doGet;
    }
});
proxy = new Proxy(Object.create(null), handler);
assert.throws(TypeError, () => proxy == 0);
assert.compareArray(log, [Symbol.toPrimitive, "valueOf", "toString"]);
