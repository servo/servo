// Copyright 2021 the V8 project authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: Value when private name describes a method
info: |
  7. Let privateName be ? GetValue(privateNameBinding).
  8. Assert: privateName is a Private Name.
  [...]
  10. Else,
      a. Assert: privateName.[[Kind]] is "method" or "accessor".
      b. If PrivateBrandCheck(rval, privateName) is not an abrupt completion,
         then return true.
  11. Return false.
esid: sec-relational-operators-runtime-semantics-evaluation
features: [class-methods-private, class-fields-private-in]
---*/

let count = 0;

class Class {
  #method() {
    count += 1;
  }

  static isNameIn(value) {
    return #method in value;
  }
}

assert.sameValue(Class.isNameIn({}), false);
assert.sameValue(Class.isNameIn(new Class()), true);
assert.sameValue(count, 0);
