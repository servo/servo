// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    The form (?= Disjunction ) specifies a zero-width positive lookahead.
    In order for it to succeed, the pattern inside Disjunction must match at the current position, but the current position is not advanced before matching the sequel.
    If Disjunction can match at the current position in several ways, only the first one is tried
es5id: 15.10.2.8_A1_T3
description: >
    Execute /[Jj]ava([Ss]cript)?(?=\:)/.exec("just Javascript: the way
    af jedi") and check results
---*/

var __executed = /[Jj]ava([Ss]cript)?(?=\:)/.exec("just Javascript: the way af jedi");

var __expected = ["Javascript", "script"];
__expected.index = 5;
__expected.input = "just Javascript: the way af jedi";

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
