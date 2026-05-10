// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    The production CharacterClassEscape :: w evaluates by returning the set of characters containing the sixty-three characters:
    a - z, A - Z, 0 - 9, _
es5id: 15.10.2.12_A3_T5
description: non-w
---*/

var non_w = "\f\n\r\t\v~`!@#$%^&*()-+={[}]|\\:;'<,>./? " + '"';

assert.sameValue(
  /\w/.exec(non_w),
  null,
  '/w/.exec(""fnrtv~`!@#$%^&*()-+={[}]|:;\'<,>./? " + \'"\'") must return null'
);

var non_W = "_0123456789_abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ";
var regexp_w = /\w/g;
var k = 0;
while (regexp_w.exec(non_W) !== null) {
   k++;
}

assert.sameValue(non_W.length, k, 'The value of non_W.length is expected to equal the value of k');
