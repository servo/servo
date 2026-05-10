// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    The production Atom :: . evaluates as follows:
    i) Let A be the set of all characters except the four line terminator characters <LF>, <CR>, <LS>, or <PS>
    ii) Call CharacterSetMatcher(A, false) and return its Matcher result
es5id: 15.10.2.8_A4_T3
description: Execute /.*a.* /.exec("this is a test") and check results
---*/

var __string = "this is a test";
var __executed = /.*a.*/.exec(__string);

var __expected = ["this is a test"];
__expected.index = 0;
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
