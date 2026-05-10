// Copyright 2021 the V8 project authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: Parsing observes the `Yield` production parameter when present
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
---*/

class C {
  #field;

  static *isNameIn() {
    return #field in (yield);
  }
}

let iter1 = C.isNameIn();
iter1.next();
assert.sameValue(iter1.next(new C()).value, true);

let iter2 = C.isNameIn();
iter2.next();
assert.sameValue(iter2.next({}).value, false);
