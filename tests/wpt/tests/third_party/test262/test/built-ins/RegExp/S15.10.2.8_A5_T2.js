// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    In case-insignificant matches all characters are implicitly converted to
    upper case immediately before they are compared
es5id: 15.10.2.8_A5_T2
description: Execute /[a-z]+/.exec("ABC def ghi") and check results
---*/

var __string = "ABC def ghi";
var __executed = /[a-z]+/.exec(__string);

var __expected = ["def"];
__expected.index = 4;
__expected.input = __string;

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
