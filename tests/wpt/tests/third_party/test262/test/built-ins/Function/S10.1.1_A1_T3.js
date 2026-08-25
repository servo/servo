// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    Program functions are defined in source text by a FunctionDeclaration or created dynamically either
    by using a FunctionExpression or by using the built-in Function object as a constructor
es5id: 10.1.1_A1_T3
description: >
    Creating function dynamically by using the built-in Function
    object as a constructor
---*/

var x = new function f1() {
  return 1;
};

assert.sameValue(
  typeof(x.constructor),
  "function",
  'The value of `typeof(x.constructor)` is expected to be "function"'
);
