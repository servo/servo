// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    The production CharacterClass :: [ ^ ClassRanges ] evaluates by
    evaluating ClassRanges to  obtain a CharSet and returning that CharSet
    and the boolean true
es5id: 15.10.2.13_A2_T1
description: Execute /[^]a/m.exec("a\naba") and check results
---*/

var __executed = /[^]a/m.exec("a\naba");

var __expected = ["\na"];
__expected.index = 1;
__expected.input = "a\naba";

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
