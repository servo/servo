// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    The production QuantifierPrefix :: { DecimalDigits , }evaluates as follows:
    i) Let i be the MV of DecimalDigits
    ii) Return the two results i and \infty
es5id: 15.10.2.7_A6_T6
description: Execute /x{1,2}x{1,}/.exec("xxxxxxx") and check results
---*/

var __executed = /x{1,2}x{1,}/.exec("xxxxxxx");

var __expected = ["xxxxxxx"];
__expected.index = 0;
__expected.input = "xxxxxxx";

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
