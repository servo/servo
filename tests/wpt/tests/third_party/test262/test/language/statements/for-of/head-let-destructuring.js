// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-for-in-and-for-of-statements
es6id: 13.7.5
description: >
  The token sequence `let [`is interpreted as the beginning of a destructuring
  binding pattern
info: |
  Syntax

  IterationStatement[Yield, Return]:

    for ( [lookahead ≠ let]LeftHandSideExpression[?Yield] of
      AssignmentExpression[+In, ?Yield] ) Statement[?Yield, ?Return]

    for ( ForDeclaration[?Yield] of AssignmentExpression[+In, ?Yield] )
      Statement[?Yield, ?Return]
---*/

var value;

for ( let[x] of [[34]] ) {
  value = x;
}

assert.sameValue(typeof x, 'undefined', 'binding is block-scoped');
assert.sameValue(value, 34);
