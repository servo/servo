// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    RegExp.prototype.exec behavior depends on global property.
    If global is true next exec calling start to match from lastIndex position
es5id: 15.10.6.2_A4_T5
description: >
    Call first exec, then set re.lastIndex = x and again call exec,
    where x is undefined variable
---*/

var __re = /(?:ab|cd)\d?/g;

var __executed = __re.exec("aacd2233ab12nm444ab42");

var __expected = ["cd2"];
__expected.index = 2;
__expected.input = "aacd2233ab12nm444ab42";

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

__re.lastIndex = x;

__executed = __re.exec("aacd2233ab12nm444ab42");

__expected = ["cd2"];
__expected.index = 2;
__expected.input = "aacd2233ab12nm444ab42";

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

var x;
