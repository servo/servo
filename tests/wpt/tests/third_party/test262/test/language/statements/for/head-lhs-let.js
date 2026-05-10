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

    for ( [lookahead ∉ { let [ } ] Expression[~In, ?Yield]opt ;
      Expression[+In, ?Yield]opt ; Expression[+In, ?Yield]opt )
      Statement[?Yield, ?Return]

    for ( LexicalDeclaration[~In, ?Yield] Expression[+In, ?Yield]opt ;
      Expression[+In, ?Yield]opt) Statement[?Yield, ?Return]
flags: [noStrict]
---*/

var let;

let = 1;
for ( let; ; )
  break;

assert.sameValue(let, 1, 'IdentifierReference');

let = 2;
for ( let = 3; ; )
  break;

assert.sameValue(let, 3, 'AssignmentExpression');

let = 4;
for ( [let][0]; ; )
  break;

assert.sameValue(let, 4, 'MemberExpression');
