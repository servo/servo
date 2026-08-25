// Copyright 2021 the V8 project authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: Rejected as assignment target
info: |
  12.10.5 Static Semantics: AllPrivateIdentifiersValid

  AllPrivateIdentifiersValid is an abstract operation which takes names as an argument.

  RelationalExpression:PrivateIdentifierinShiftExpression

  1. If StringValue of PrivateIdentifier is in names, return true.
  2. Return false.

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
    #field in {} = 0;
  }
}
