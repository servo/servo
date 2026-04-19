// Copyright (C) 2021 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: >
  Can't nest `in` expressions when the left-hand side is PrivateIdentifier.
info: |
  Syntax
    RelationalExpression[In, Yield, Await]:
    [...]
    [+In]PrivateIdentifier in ShiftExpression[?Yield, ?Await]
esid: sec-relational-operators
negative:
  phase: parse
  type: SyntaxError
features: [class-fields-private, class-fields-private-in]
---*/

$DONOTEVALUATE();

class C {
  #field;

  constructor() {
    #field in #field in this;
  }
}
