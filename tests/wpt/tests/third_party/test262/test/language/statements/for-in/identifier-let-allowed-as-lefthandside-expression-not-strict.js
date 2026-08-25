// Copyright (C) 2011 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: >
    identifier "let" allowed as lefthandside expression
esid: sec-iteration-statements
info: |
  for ( [ lookahead ∉ { let [ } ] LeftHandSideExpression [?Yield, ?Await] in 
    Expression[+In, ?Yield, ? Await]) Statement[?Yield, ?Await, ?Return]
flags: [noStrict]
---*/
for (let in {}) { }
