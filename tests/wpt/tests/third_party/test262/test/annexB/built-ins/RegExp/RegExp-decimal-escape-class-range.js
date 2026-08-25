// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    The production CharacterClass :: [ [lookahead \notin {^}] ClassRanges ]
    evaluates by evaluating ClassRanges to obtain a CharSet and returning
    that CharSet and the boolean false
es5id: 15.10.2.13_A1_T16
es6id: B.1.4
description: >
    Execute /[\d][\12-\14]{1,}[^\d]/.exec("line1\n\n\n\n\nline2") and
    check results
---*/

var __executed = /[\d][\12-\14]{1,}[^\d]/.exec("line1\n\n\n\n\nline2");

var __expected = ["1\n\n\n\n\nl"];
__expected.index = 4;
__expected.input = "line1\n\n\n\n\nline2";

assert.sameValue(__executed.length, __expected.length, '.length');
assert.sameValue(__executed.index, __expected.index, '.index');
assert.sameValue(__executed.input, __expected.input, '.input');

//CHECK#4
for(var index=0; index < __expected.length; index++) {
  assert.sameValue(__executed[index], __expected[index], 'index: ' + index);
}
