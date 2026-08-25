// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    The production Assertion :: ^ evaluates by returning an internal
    AssertionTester closure that takes a State argument x and performs the ...
es5id: 15.10.2.6_A2_T3
description: >
    Execute /^p[a-z]/.exec("pairs\nmakes\tdouble\npesos") and check
    results
---*/

var __executed = /^p[a-z]/.exec("pairs\nmakes\tdouble\npesos");

var __expected = ["pa"];
__expected.index = 0;
__expected.input = "pairs\nmakes\tdouble\npesos";

assert.sameValue(
  __executed.length,
  __expected.length,
  'The value of __executed.length is expected to equal the value of __expected.length'
);

assert.sameValue(
  __executed.index,
  __expected.index,
  'The value of __executed.index is expected to equal the value of __expected.index'
);

assert.sameValue(
  __executed.input,
  __expected.input,
  'The value of __executed.input is expected to equal the value of __expected.input'
);

for(var index=0; index<__expected.length; index++) {
  assert.sameValue(
    __executed[index],
    __expected[index],
    'The value of __executed[index] is expected to equal the value of __expected[index]'
  );
}
