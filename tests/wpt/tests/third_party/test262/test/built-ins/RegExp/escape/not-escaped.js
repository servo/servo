// Copyright (C) 2024 Leo Balter. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-regexp.escape
description: Numbers and alphabetic characters are not escaped
info: |
  RegExp.escape ( string )

  This method produces a new string in which certain characters have been escaped.
  These characters are: . * + ? ^ $ | ( ) [ ] { } \

features: [RegExp.escape]
---*/

const asciiletter = 'abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ';
const decimaldigits = '0123456789';

assert.sameValue(RegExp.escape(''), '', 'the empty string is not escaped');

asciiletter.split('').forEach(char => {
  assert.sameValue(RegExp.escape(`.${char}`), `\\.${char}`, `ASCII letter ${char} is not escaped`);
});

assert.sameValue(RegExp.escape(`.${asciiletter}`), `\\.${asciiletter}`, 'ASCII letters are not escaped');

decimaldigits.split('').forEach(char => {
  assert.sameValue(RegExp.escape(`.${char}`), `\\.${char}`, `decimal digit ${char} is not escaped`);
});

assert.sameValue(RegExp.escape(`.${decimaldigits}`), `\\.${decimaldigits}`, 'decimal digits are not escaped');

assert.sameValue(RegExp.escape('.a1b2c3D4E5F6'), '\\.a1b2c3D4E5F6', 'mixed string with ASCII letters and decimal digits is not escaped');
