// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
flags:
  - noStrict
description: |
  pending
esid: pending
---*/

function createProxy(proxyTarget) {
  var {proxy, revoke} = Proxy.revocable(proxyTarget, new Proxy({}, {
    get(target, propertyKey, receiver) {
      revoke();
    }
  }));
  return proxy;
}

var obj;

// [[GetPrototypeOf]]
assert.sameValue(Object.getPrototypeOf(createProxy({})), Object.prototype);
assert.sameValue(Object.getPrototypeOf(createProxy([])), Array.prototype);

// [[SetPrototypeOf]]
obj = {};
Object.setPrototypeOf(createProxy(obj), Array.prototype);
assert.sameValue(Object.getPrototypeOf(obj), Array.prototype);

// [[IsExtensible]]
assert.sameValue(Object.isExtensible(createProxy({})), true);
assert.sameValue(Object.isExtensible(createProxy(Object.preventExtensions({}))), false);

// [[PreventExtensions]]
obj = {};
Object.preventExtensions(createProxy(obj));
assert.sameValue(Object.isExtensible(obj), false);

// [[GetOwnProperty]]
assert.sameValue(Object.getOwnPropertyDescriptor(createProxy({}), "a"), undefined);
assert.sameValue(Object.getOwnPropertyDescriptor(createProxy({a: 5}), "a").value, 5);

// [[DefineOwnProperty]]
obj = {};
Object.defineProperty(createProxy(obj), "a", {value: 5});
assert.sameValue(obj.a, 5);

// [[HasProperty]]
assert.sameValue("a" in createProxy({}), false);
assert.sameValue("a" in createProxy({a: 5}), true);

// [[Get]]
assert.sameValue(createProxy({}).a, undefined);
assert.sameValue(createProxy({a: 5}).a, 5);

// [[Set]]
assert.throws(TypeError, () => createProxy({}).a = 0);
assert.throws(TypeError, () => createProxy({a: 5}).a = 0);

// [[Delete]]
assert.sameValue(delete createProxy({}).a, true);
assert.sameValue(delete createProxy(Object.defineProperty({}, "a", {configurable: false})).a, false);

// [[OwnPropertyKeys]]
assert.sameValue(Object.getOwnPropertyNames(createProxy({})).length, 0);
assert.sameValue(Object.getOwnPropertyNames(createProxy({a: 5})).length, 1);

// [[Call]]
assert.sameValue(createProxy(function() { return "ok" })(), "ok");

// [[Construct]]
// This throws because after the "construct" trap on the proxy is consulted,
// OrdinaryCreateFromConstructor (called because the |q| function's
// [[ConstructorKind]] is "base" per FunctionAllocate) accesses
// |new.target.prototype| to create the |this| for the construct operation, that
// would be returned if |return obj;| didn't override it.
assert.throws(TypeError, () => new (createProxy(function q(){ return obj; })));
