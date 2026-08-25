// Copyright (C) 2017 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-while-statement
description: >
  ExpressionStatement doesn't have a lookahead restriction for `let <binding-identifier>`.
info: |
  ExpressionStatement[Yield, Await] :
    [lookahead ∉ { {, function, async [no LineTerminator here] function, class, let [ }]
    Expression[+In, ?Yield, ?Await] ;
flags: [noStrict]
---*/

while (false) let // ASI
x = 1;
