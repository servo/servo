// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    An escape sequence of the form \ followed by a nonzero decimal number n
    matches the result of the nth set of capturing parentheses (see
    15.10.2.11)
es5id: 15.10.2.9_A1_T5
description: Execute /(a*)b\1+/.exec("baaac") and check results
---*/

var __executed = /(a*)b\1+/.exec("baaac");

var __expected = ["b", ""];
__expected.index = 0;
__expected.input = "baaac";

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
