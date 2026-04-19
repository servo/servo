// Copyright (C) 2019 Caio Lima. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: Successfully access private method on Proxy objects without using [[Get]]
esid: sec-getvalue
info: |
  GetValue(V)
    ...
    5. If IsPropertyReference(V), then
      ...
      b. If IsPrivateReference(V), then
        i. Let env be the running execution context's PrivateNameEnvironment.
        ii. Let field be ? ResolveBinding(GetReferencedName(V), env).
        iii. Assert: field is a Private Name.
        iv. Return ? PrivateFieldGet(field, base).
      c. Return ? base.[[Get]](GetReferencedName(V), GetThisValue(V)).
  PrivateFieldGet(P, O)
    ...
    4. Perform ? PrivateBrandCheck(O, P).
    5. If P.[[Kind]] is "method",
      a. Return P.[[Value]].
      ...
includes: [compareArray.js]
features: [class, class-methods-private, Proxy]
---*/

let arr = [];

class ProxyBase {
  constructor() {
    return new Proxy(this, {
      get: function (obj, prop) {
        arr.push(prop);
        return obj[prop];
      }
    });
  }
}

class Test extends ProxyBase {
  #f() {
    return 3;
  }
  method() {
    return this.#f();
  }
}

let t = new Test();
let r = t.method();
assert.sameValue(r, 3);

assert.compareArray(arr, ['method']);

