// Copyright (C) 2019 Caio Lima. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: Public class fields initialization calls [[DefineOwnProperty]]
esid: sec-define-field
info: |
  DefineField(receiver, fieldRecord)
    ...
    8. If fieldName is a Private Name,
      a. Perform ? PrivateFieldAdd(fieldName, receiver, initValue).
    9. Else,
      a. Assert: IsPropertyKey(fieldName) is true.
      b. Perform ? CreateDataPropertyOrThrow(receiver, fieldName, initValue).
    10. Return.
includes: [compareArray.js]
features: [class, class-fields-public, Proxy]
---*/

let arr = [];
let expectedTarget = null;
function ProxyBase() {
  expectedTarget = this;
  return new Proxy(this, {
    defineProperty: function (target, key, descriptor) {
      arr.push(key);
      arr.push(descriptor.value);
      arr.push(target);
      assert.sameValue(descriptor.enumerable, true);
      assert.sameValue(descriptor.configurable, true);
      assert.sameValue(descriptor.writable, true);
      return Reflect.defineProperty(target, key, descriptor);
    }
  });
}

class Test extends ProxyBase {
  f = 3;
  g = "Test262";
}

let t = new Test();
assert.sameValue(t.f, 3);
assert.sameValue(t.g, "Test262");

assert.compareArray(arr, ["f", 3, expectedTarget, "g", "Test262", expectedTarget]);
