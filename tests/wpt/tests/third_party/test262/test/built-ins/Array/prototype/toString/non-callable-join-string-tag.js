// Copyright (C) 2021 Alexey Shvayka. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.tostring
description: >
    If "join" value is non-callable, Object.prototype.toString intrinsic is called.
info: |
    Array.prototype.toString ( )

    [...]
    2. Let func be ? Get(array, "join").
    3. If IsCallable(func) is false, set func to the intrinsic function %Object.prototype.toString%.
    4. Return ? Call(func, array).
features: [Symbol.toStringTag, Proxy, Reflect, BigInt]
---*/

assert(delete Object.prototype.toString);

assert.sameValue(Array.prototype.toString.call({ join: null }), "[object Object]");
assert.sameValue(Array.prototype.toString.call({ join: true }), "[object Object]");
assert.sameValue(Array.prototype.toString.call({ join: 0 }), "[object Object]");
assert.sameValue(Array.prototype.toString.call({ join: "join" }), "[object Object]");
assert.sameValue(Array.prototype.toString.call({ join: Symbol() }), "[object Object]");
assert.sameValue(Array.prototype.toString.call({ join: 0n }), "[object Object]");
assert.sameValue(Array.prototype.toString.call({ join: {} }), "[object Object]");

let revokeOnGet = false;
const proxyTarget = [];
var proxyObj = Proxy.revocable(proxyTarget, {
    get: (target, key, receiver) => {
        if (revokeOnGet)
            revoke();
        return Reflect.get(target, key, receiver);
    },
});
var proxy = proxyObj.proxy;
var revoke = proxyObj.revoke;

proxyTarget.join = undefined;
assert.sameValue(Array.prototype.toString.call(proxy), "[object Array]");
revokeOnGet = true;
assert.throws(TypeError, () => { Array.prototype.toString.call(proxy); });

assert.sameValue(Array.prototype.toString.call((function() { return arguments; })()), "[object Arguments]");
assert.sameValue(Array.prototype.toString.call(new Error), "[object Error]");
assert.sameValue(Array.prototype.toString.call(new Boolean), "[object Boolean]");
assert.sameValue(Array.prototype.toString.call(new Number), "[object Number]");
assert.sameValue(Array.prototype.toString.call(new String), "[object String]");
assert.sameValue(Array.prototype.toString.call(new Date), "[object Date]");
assert.sameValue(Array.prototype.toString.call(new RegExp), "[object RegExp]");
assert.sameValue(Array.prototype.toString.call(new Proxy(() => {}, {})), "[object Function]");
assert.sameValue(Array.prototype.toString.call(new Proxy(new Date, {})), "[object Object]");
assert.sameValue(Array.prototype.toString.call({ [Symbol.toStringTag]: "Foo" }), "[object Foo]");
assert.sameValue(Array.prototype.toString.call(new Map), "[object Map]");

RegExp.prototype[Symbol.toStringTag] = "Foo";
assert.sameValue(Array.prototype.toString.call(new RegExp), "[object Foo]");
Number.prototype[Symbol.toStringTag] = Object("Foo"); // ignored
assert.sameValue(Array.prototype.toString.call(new Number), "[object Number]");

Object.defineProperty(JSON, Symbol.toStringTag, { value: "Foo" });
assert.sameValue(Array.prototype.toString.call(JSON), "[object Foo]");

assert(delete Set.prototype[Symbol.toStringTag]);
assert.sameValue(Array.prototype.toString.call(new Set), "[object Object]");

Object.defineProperty(Object.prototype, Symbol.toStringTag, { get: () => { throw new Test262Error(); } });
assert.throws(Test262Error, () => { Array.prototype.toString.call({}); });
