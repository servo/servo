// Copyright (C) 2024 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-object-environment-records-hasbinding-n
description: >
  Lookups in proxy binding object for call expression.
info: |
  9.1.1.2.1 HasBinding ( N )

    1. Let bindingObject be envRec.[[BindingObject]].
    2. Let foundBinding be ? HasProperty(bindingObject, N).
    3. If foundBinding is false, return false.
    ...

features: [Proxy, Reflect]
flags: [noStrict]
includes: [compareArray.js, proxyTrapsHelper.js]
---*/

var log = [];

// Empty environment.
var env = {};

var proxy = new Proxy(env, allowProxyTraps({
  has(t, pk) {
    log.push("has:" + String(pk));
    return Reflect.has(t, pk);
  },
}));

with (proxy) {
  Object();
}

assert.compareArray(log, [
  // HasBinding, step 2.
  "has:Object",
]);
