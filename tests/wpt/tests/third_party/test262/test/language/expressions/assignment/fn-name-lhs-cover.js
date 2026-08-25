// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-assignment-operators-runtime-semantics-evaluation
description: Left-hand side as a CoverParenthesizedExpression
info: |
    AssignmentExpression[In, Yield, Await] :
        LeftHandSideExpression[?Yield, ?Await] = AssignmentExpression[?In, ?Yield, ?Await]

    1. If LeftHandSideExpression is neither an ObjectLiteral nor an
       ArrayLiteral, then
       [...]
       c. If IsAnonymousFunctionDefinition(AssignmentExpression) and
          IsIdentifierRef of LeftHandSideExpression are both true, then
          i. Let rval be NamedEvaluation of AssignmentExpression with argument
             GetReferencedName(lref).
includes: [propertyHelper.js]
---*/

var fn;

(fn) = function() {};

verifyProperty(fn, "name", {
  value: "", writable: false, enumerable: false, configurable: true
});
