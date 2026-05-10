// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    It is permissible but not necessary to have one argument for each formal
    parameter to be specified
es5id: 15.3.2.1_A2_T2
description: >
    Values of the function constructor arguments are "arg1, arg2",
    "arg3", "return arg1+arg2+arg3;"
---*/

try {
  var f = Function("arg1, arg2", "arg3", "return arg1+arg2+arg3;");
} catch (e) {
  throw new Test262Error('#1: test failed');
}

assert(f instanceof Function, 'The result of evaluating (f instanceof Function) is expected to be true');
assert.sameValue(f("AB", "BA", 1), "ABBA1", 'f(AB, BA, 1) must return "ABBA1"');
