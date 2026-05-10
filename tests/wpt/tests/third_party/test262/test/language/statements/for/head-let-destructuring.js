// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-iteration-statements
es6id: 13.7
description: >
  The token sequence `let [`is interpreted as the beginning of a destructuring
  binding pattern
info: |
  Syntax

  IterationStatement[Yield, Return]:

    for ( [lookahead ∉ { let [ } ] Expression[~In, ?Yield]opt ;
      Expression[+In, ?Yield]opt ; Expression[+In, ?Yield]opt )
      Statement[?Yield, ?Return]

    for ( LexicalDeclaration[~In, ?Yield] Expression[+In, ?Yield]opt ;
      Expression[+In, ?Yield]opt) Statement[?Yield, ?Return]
---*/

var value;

for ( let[x] = [23]; ; ) {
  value = x;
  break;
}

assert.sameValue(typeof x, 'undefined', 'binding is block-scoped');
assert.sameValue(value, 23);
