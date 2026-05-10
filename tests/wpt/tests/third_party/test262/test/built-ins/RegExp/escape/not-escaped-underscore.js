// Copyright (C) 2024 Leo Balter. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-encodeforregexescape
description: Ensures the underscore character is not escaped
info: |
  RegExp.escape ( string )

  This method produces a new string in which certain characters have been escaped.
  These characters are: . * + ? ^ $ | ( ) [ ] { } \ , - = < > # & ! % : ; @ ~ ' ` " and white space or line terminators.
features: [RegExp.escape]
---*/

assert.sameValue(RegExp.escape('_'), '_', 'Single underscore character is not escaped');
assert.sameValue(RegExp.escape('__'), '__', 'Thunderscore character is not escaped');
assert.sameValue(RegExp.escape('hello_world'), '\\x68ello_world', 'String starting with ASCII letter and containing underscore is correctly escaped');
assert.sameValue(RegExp.escape('1_hello_world'), '\\x31_hello_world', 'String starting with digit and containing underscore is correctly escaped');
assert.sameValue(RegExp.escape('a_b_c'), '\\x61_b_c', 'String starting with ASCII letter and containing multiple underscores is correctly escaped');
assert.sameValue(RegExp.escape('3_b_4'), '\\x33_b_4', 'String starting with digit and containing multiple underscores is correctly escaped');
assert.sameValue(RegExp.escape('_hello'), '_hello', 'String starting with underscore and containing other characters is not escaped');
assert.sameValue(RegExp.escape('_1hello'), '_1hello', 'String starting with underscore and digit is not escaped');
assert.sameValue(RegExp.escape('_a_1_2'), '_a_1_2', 'String starting with underscore and mixed characters is not escaped');
