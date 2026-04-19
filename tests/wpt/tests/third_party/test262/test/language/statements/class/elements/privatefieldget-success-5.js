// Copyright (C) 2017 Valerie Young. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: Successfully access private field
esid: sec-getvalue
info: |
  GetValue ( V )
    ...
    5. If IsPropertyReference(V), then
      ...
      b. If IsPrivateReference(V), then
        i. Let env be the running execution context's PrivateNameEnvironment.
        ii. Let field be ? ResolveBinding(GetReferencedName(V), env).
        iii. Assert: field is a Private Name.
        iv. Return ? PrivateFieldGet(field, base).

  PrivateFieldGet (P, O )
    1. Assert: P is a Private Name value.
    2. If O is not an object, throw a TypeError exception.
    3. Let entry be PrivateFieldFind(P, O).
    4. If entry is empty, throw a TypeError exception.
    5. Return entry.[[PrivateFieldValue]].

features: [class, class-fields-private]
---*/


class C {
  #x = 42;
  f() {
    return this.#x;
  }
}

var c1 = new C();
var c2 = new C();
var value = c2.f.call(c1);

assert.sameValue(value, 42);
