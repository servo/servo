// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-for-in-and-for-of-statements
es6id: 13.7.5
description: >
  The `let` token is disallowed when not followed by a `[` token
info: |
  Syntax

  IterationStatement[Yield, Return]:

    for ( [lookahead ≠ let]LeftHandSideExpression[?Yield] of
      AssignmentExpression[+In, ?Yield] ) Statement[?Yield, ?Return]

    for ( ForDeclaration[?Yield] of AssignmentExpression[+In, ?Yield] )
      Statement[?Yield, ?Return]
negative:
  phase: parse
  type: SyntaxError
---*/

$DONOTEVALUATE();

for ( let of [] ) ;
