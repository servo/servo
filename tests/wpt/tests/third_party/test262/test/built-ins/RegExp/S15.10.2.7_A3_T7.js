// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    The production QuantifierPrefix :: + evaluates by returning the two
    results 1 and \infty
es5id: 15.10.2.7_A3_T7
description: >
    Execute /[a-z]+(\d+)/.exec("x 2 ff 55 x2 as1 z12 abc12.0") and
    check results
---*/

var __executed = /[a-z]+(\d+)/.exec("x 2 ff 55 x2 as1 z12 abc12.0");

var __expected = ["x2","2"];
__expected.index = 10;
__expected.input = "x 2 ff 55 x2 as1 z12 abc12.0";

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
