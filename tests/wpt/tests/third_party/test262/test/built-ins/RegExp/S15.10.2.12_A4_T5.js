// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    The production CharacterClassEscape :: W evaluates by returning the set of all characters not
    included in the set returned by CharacterClassEscape :: w
es5id: 15.10.2.12_A4_T5
description: non-w
---*/

var non_w = "\f\n\r\t\v~`!@#$%^&*()-+={[}]|\\:;'<,>./? " + '"';
var regexp_W = /\W/g;
var k = 0;
while (regexp_W.exec(non_w) !== null) {
   k++;
}

assert.sameValue(non_w.length, k, 'The value of non_w.length is expected to equal the value of k');

var non_W = "_0123456789_abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ";

assert.sameValue(
  /\W/.exec(non_W),
  null,
  '/W/.exec(""_0123456789_abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ"") must return null'
);
