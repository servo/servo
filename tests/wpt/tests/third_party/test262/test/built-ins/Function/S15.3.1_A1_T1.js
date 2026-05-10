// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    The function call Function(…) is equivalent to the object creation expression
    new Function(…) with the same arguments.
es5id: 15.3.1_A1_T1
description: Create simple functions and check returned values
---*/

var f = Function("return arguments[0];");

assert(f instanceof Function, 'The result of evaluating (f instanceof Function) is expected to be true');
assert.sameValue(f(1), 1, 'f(1) must return 1');

var g = new Function("return arguments[0];");


assert(g instanceof Function, 'The result of evaluating (g instanceof Function) is expected to be true');
assert.sameValue(g("A"), "A", 'g("A") must return "A"');
assert.sameValue(g("A"), f("A"), 'g("A") must return the same value returned by f("A")');
