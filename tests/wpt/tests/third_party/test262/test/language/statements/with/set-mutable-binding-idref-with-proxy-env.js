// Copyright (C) 2024 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-object-environment-records-setmutablebinding-n-v-s
description: >
  Lookups in proxy binding object for identifier reference.
info: |
  9.1.1.2.1 HasBinding ( N )

    1. Let bindingObject be envRec.[[BindingObject]].
    2. Let foundBinding be ? HasProperty(bindingObject, N).
    3. If foundBinding is false, return false.
    ...
    5. Let unscopables be ? Get(bindingObject, %Symbol.unscopables%).
    ...
    7. Return true.

  9.1.1.2.5 SetMutableBinding ( N, V, S )

    1. Let bindingObject be envRec.[[BindingObject]].
    2. Let stillExists be ? HasProperty(bindingObject, N).
    ...
    4. Perform ? Set(bindingObject, N, V, S).
    ...

  10.1.9.2 OrdinarySetWithOwnDescriptor ( O, P, V, Receiver, ownDesc )

    ...
    2. If IsDataDescriptor(ownDesc) is true, then
      ...
      c. Let existingDescriptor be ? Receiver.[[GetOwnProperty]](P).
      d. If existingDescriptor is not undefined, then
        ...
        iv. Return ? Receiver.[[DefineOwnProperty]](P, valueDesc).
      ...

features: [Proxy, Reflect]
flags: [noStrict]
includes: [compareArray.js, proxyTrapsHelper.js]
---*/

var log = [];

// Environment contains referenced binding.
var env = {
  p: 0,
};

var proxy = new Proxy(env, allowProxyTraps({
  has(t, pk) {
    log.push("has:" + String(pk));
    return Reflect.has(t, pk);
  },
  get(t, pk, r) {
    log.push("get:" + String(pk));
    return Reflect.get(t, pk, r);
  },
  set(t, pk, v, r) {
    log.push("set:" + String(pk));
    return Reflect.set(t, pk, v, r);
  },
  getOwnPropertyDescriptor(t, pk) {
    log.push("getOwnPropertyDescriptor:" + String(pk));
    return Reflect.getOwnPropertyDescriptor(t, pk);
  },
  defineProperty(t, pk, d) {
    log.push("defineProperty:" + String(pk));
    return Reflect.defineProperty(t, pk, d);
  },
}));

with (proxy) {
  p = 1;
}

assert.compareArray(log, [
  // HasBinding, step 2.
  "has:p",

  // HasBinding, step 5.
  "get:Symbol(Symbol.unscopables)",

  // SetMutableBinding, step 2.
  "has:p",

  // SetMutableBinding, step 4.
  "set:p",
  "getOwnPropertyDescriptor:p",
  "defineProperty:p",
]);
