// Copyright 2021 the V8 project authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: Algorithm interrupted by unresolvable reference
info: |
  1. Let privateIdentifier be the StringValue of PrivateIdentifier.
  2. Let rref be the result of evaluating ShiftExpression.
  3. Let rval be ? GetValue(rref).
esid: sec-relational-operators-runtime-semantics-evaluation
features: [class-fields-private, class-fields-private-in]
---*/

let caught = null;

class C {
  #field;
  constructor() {
    try {
      #field in test262unresolvable;
    } catch (error) {
      caught = error;
    }
  }
}

new C();

assert.notSameValue(caught, null);
assert.sameValue(caught.constructor, ReferenceError);
