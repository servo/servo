// Copyright (C) 2021 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: >
  Private identifiers aren't valid simple assignment references.
info: |
  Syntax
    for ( LeftHandSideExpression in Expression ) Statement 
esid: sec-for-in-and-for-of-statements-static-semantics-early-errors
negative:
  phase: parse
  type: SyntaxError
features: [class-fields-private, class-fields-private-in]
---*/

$DONOTEVALUATE();

class C {
  #field;

  m() {
    for (#field in []) ;
  }
}
