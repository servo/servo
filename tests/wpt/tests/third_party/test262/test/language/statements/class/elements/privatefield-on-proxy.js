// Copyright (C) 2019 Caio Lima. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: Sucessyfully get private reference without using [[Get]]
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
    1. Assert: P is a Private Name value.
    2. If O is not an object, throw a TypeError exception.
    3. Let entry be PrivateFieldFind(P, O).
    4. If entry is empty, throw a TypeError exception.
    5. Return entry.[[PrivateFieldValue]].
includes: [compareArray.js]
features: [class, class-fields-private, Proxy]
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
  #f = 3;
  method() {
    return this.#f;
  }
}

let t = new Test();
let r = t.method();
assert.sameValue(r, 3);

assert.compareArray(arr, ['method']);

