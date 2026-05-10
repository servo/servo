// Copyright 2019 Google, Inc.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: prod-OptionalExpression
description: >
  should not suppress error if super called on class with no base
info: |
  Left-Hand-Side Expressions
    OptionalExpression:
      SuperCall OptionalChain
features: [optional-chaining]
negative:
  phase: parse
  type: SyntaxError
---*/

$DONOTEVALUATE();

class C {
  constructor () {
    super()?.a;
  }
}
