// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    Since assertion evaluating do not change endIndex repetition of assertion
    does the same result
es5id: 15.10.2.6_A5_T1
description: Execute /^^^^^^^robot$$$$/.exec("robot") and check results
---*/

var __executed = /^^^^^^^robot$$$$/.exec("robot");

var __expected = ["robot"];
__expected.index = 0;
__expected.input = "robot";

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
