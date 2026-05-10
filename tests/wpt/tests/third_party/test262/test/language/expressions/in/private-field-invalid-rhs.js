// Copyright 2021 the V8 project authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: Syntactic grammar restricts right-hand side
info: |
  Syntax
    RelationalExpression[In, Yield, Await]:
    [...]
    [+In]PrivateIdentifier in ShiftExpression[?Yield, ?Await]
esid: sec-relational-operators-runtime-semantics-evaluation
negative:
  phase: parse
  type: SyntaxError
features: [class-fields-private, class-fields-private-in]
---*/

$DONOTEVALUATE();

class C {
  #field;

  constructor() {
    #field in () => {};
  }
}
