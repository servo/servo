// Copyright (C) 2020 Alexey Shvayka. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-proxy-object-internal-methods-and-internal-slots-set-p-v-receiver
description: >
  If "set" trap is missing, the call is properly forwarded to ordinary
  [[Set]] that invokes [[GetOwnProperty]] and [[DefineOwnProperty]] methods
  on receiver (that is a Proxy itself) every time it is called.
info: |
  [[Set]] ( P, V, Receiver )

  [...]
  6. Let trap be ? GetMethod(handler, "set").
  7. If trap is undefined, then
    a. Return ? target.[[Set]](P, V, Receiver).

  OrdinarySetWithOwnDescriptor ( O, P, V, Receiver, ownDesc )

  [...]
  3. If IsDataDescriptor(ownDesc) is true, then
    [...]
    c. Let existingDescriptor be ? Receiver.[[GetOwnProperty]](P).
    d. If existingDescriptor is not undefined, then
      [...]
      iii. Let valueDesc be the PropertyDescriptor { [[Value]]: V }.
      iv. Return ? Receiver.[[DefineOwnProperty]](P, valueDesc).
    e. Else,
      i. Assert: Receiver does not currently have a property P.
      ii. Return ? CreateDataProperty(Receiver, P, V).
  [...]

  [[DefineOwnProperty]] ( P, Desc )

  [...]
  9. Let booleanTrapResult be ! ToBoolean(? Call(trap, handler, « target, P, descObj »)).
  [...]
  17. Return true.
features: [Proxy, Reflect]
includes: [compareArray.js]
---*/

var getOwnPropertyKeys = [];
var definePropertyKeys = [];

var p = new Proxy({
  foo: 1,
}, {
  getOwnPropertyDescriptor: function(target, key) {
    getOwnPropertyKeys.push(key);
    return Reflect.getOwnPropertyDescriptor(target, key);
  },
  defineProperty: function(target, key, desc) {
    definePropertyKeys.push(key);
    return Reflect.defineProperty(target, key, desc);
  },
});

p["foo"] = 2;
p.foo = 2;
p.foo = 2;

assert.compareArray(getOwnPropertyKeys, ["foo", "foo", "foo"],
  "getOwnPropertyDescriptor: key present on [[ProxyTarget]]");
assert.compareArray(definePropertyKeys, ["foo", "foo", "foo"],
  "defineProperty: key present on [[ProxyTarget]]");

getOwnPropertyKeys = [];
definePropertyKeys = [];

p.bar = 3;
p["bar"] = 3;
p.bar = 3;

assert.compareArray(getOwnPropertyKeys, ["bar", "bar", "bar"],
  "getOwnPropertyDescriptor: key absent on [[ProxyTarget]]");
assert.compareArray(definePropertyKeys, ["bar", "bar", "bar"],
  "defineProperty: key absent on [[ProxyTarget]]");
