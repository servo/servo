// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    RegExp.prototype.exec behavior depends on global property.
    Let global is true and let I = If ToInteger(lastIndex).
    Then if I<0 orI>length then set lastIndex to 0 and return null
es5id: 15.10.6.2_A5_T1
description: >
    First call /(?:ab|cd)\d?/g.exec("aac1dz2233a1bz12nm444ab42"), and
    then First call /(?:ab|cd)\d?/g.exec("aacd22")
---*/

var __re = /(?:ab|cd)\d?/g;
var __executed = __re.exec("aac1dz2233a1bz12nm444ab42");

var __expected = ["ab4"];
__expected.index = 21;
__expected.input = "aac1dz2233a1bz12nm444ab42";

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

__executed = __re.exec("aacd22");

assert(!__executed, 'The value of !__executed is expected to be true');
assert.sameValue(__re.lastIndex, 0, 'The value of __re.lastIndex is expected to be 0');
