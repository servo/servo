// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-iteration-statements
es6id: 13.7
description: >
  The `let` token is interpreted as an Identifier when it is not followed by a
  `[` token
info: |
  Syntax

  IterationStatement[Yield, Return]:

    for ( [lookahead ∉ { let [ } ] LeftHandSideExpression[?Yield] in
      Expression[+In, ?Yield] ) Statement[?Yield, ?Return]

    for ( ForDeclaration[?Yield] in Expression[+In, ?Yield] )
      Statement[?Yield, ?Return]
flags: [noStrict]
---*/

var obj = Object.create(null);
var let, value;

obj.key = 1;

for ( let in obj ) ;

assert.sameValue(let, 'key', 'IdentifierReference');

Object.defineProperty(Array.prototype, '1', {
  set: function(param) {
    value = param;
  }
});
for ( [let][1] in obj ) ;

assert.sameValue(value, 'key', 'MemberExpression');
