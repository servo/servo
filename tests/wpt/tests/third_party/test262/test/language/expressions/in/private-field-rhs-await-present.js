// Copyright 2021 the V8 project authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: Parsing observes the `Await` production parameter when present
info: |
  Syntax
    RelationalExpression[In, Yield, Await]:
    [...]
    [+In]PrivateIdentifier in ShiftExpression[?Yield, ?Await]

  [...]

  1. Let privateIdentifier be the StringValue of PrivateIdentifier.
  2. Let rref be the result of evaluating ShiftExpression.
  3. Let rval be ? GetValue(rref).
  4. If Type(rval) is not Object, throw a TypeError exception.
esid: sec-relational-operators-runtime-semantics-evaluation
features: [class-fields-private, class-fields-private-in]
flags: [async]
---*/

class C {
  #field;

  static async isNameIn(value) {
    return #field in await(value);
  }
}

C.isNameIn(new C())
  .then(function(result) {
    assert.sameValue(result, true);

    return C.isNameIn({});
  })
  .then(function(result) {
    assert.sameValue(result, false);
  }).then($DONE, $DONE);
