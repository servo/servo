// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: >
    White Space between LeftHandSideExpression and "=" or between "=" and
    AssignmentExpression is allowed
es5id: 11.13.1_A1
---*/

var x;

x
=
true;

if (x !== true) {
  throw new Test262Error('#6: (x\\u000A=\\u000Atrue) === true');
}
