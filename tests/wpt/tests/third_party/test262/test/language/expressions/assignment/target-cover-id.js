// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-assignment-operators-static-semantics-early-errors
es6id: 12.14.1
es5id: 11.1.6_A3_T5
description: Applied to a "covered" IdentifierReference
info: |
  AssignmentExpression : LeftHandSideExpression = AssignmentExpression

  - It is an early Reference Error if LeftHandSideExpression is neither an
    ObjectLiteral nor an ArrayLiteral and IsValidSimpleAssignmentTarget of
    LeftHandSideExpression is false.

  Static Semantics: IsValidSimpleAssignmentTarget

  IdentifierReference : Identifier

  1. If this IdentifierReference is contained in strict mode code and
     StringValue of Identifier is "eval" or "arguments", return false.
  2. Return true.
---*/

var x;

(x) = 1;

assert.sameValue(x, 1);
