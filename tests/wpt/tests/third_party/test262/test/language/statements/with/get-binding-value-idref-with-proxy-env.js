// Copyright (C) 2024 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-object-environment-records-getbindingvalue-n-s
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

  9.1.1.2.6 GetBindingValue ( N, S )

    1. Let bindingObject be envRec.[[BindingObject]].
    2. Let value be ? HasProperty(bindingObject, N).
    3. If value is false, then
      a. If S is false, return undefined; otherwise throw a ReferenceError exception.
    4. Return ? Get(bindingObject, N).

features: [Proxy, Reflect]
flags: [noStrict]
includes: [compareArray.js, proxyTrapsHelper.js]
---*/

var log = [];

// Environment contains referenced binding.
var env = {
  Object,
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
}));

with (proxy) {
  Object;
}

assert.compareArray(log, [
  // HasBinding, step 2.
  "has:Object",

  // HasBinding, step 5.
  "get:Symbol(Symbol.unscopables)",

  // GetBindingValue, step 2.
  "has:Object",

  // GetBindingValue, step 4.
  "get:Object",
]);
