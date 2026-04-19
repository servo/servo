// Copyright (C) 2024 Leo Balter. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-regexp.escape
description: Escaped characters (mixed assertions)
info: |
  RegExp.escape ( string )

  This method produces a new string in which certain characters have been escaped.
  These characters are: . * + ? ^ $ | ( ) [ ] { } \

features: [RegExp.escape]
---*/

assert.sameValue(RegExp.escape('.a.b'), '\\.a\\.b', 'mixed string with dot character is escaped correctly');
assert.sameValue(RegExp.escape('.1+2'), '\\.1\\+2', 'mixed string with plus character is escaped correctly');
assert.sameValue(RegExp.escape('.a(b)c'), '\\.a\\(b\\)c', 'mixed string with parentheses is escaped correctly');
assert.sameValue(RegExp.escape('.a*b+c'), '\\.a\\*b\\+c', 'mixed string with asterisk and plus characters is escaped correctly');
assert.sameValue(RegExp.escape('.a?b^c'), '\\.a\\?b\\^c', 'mixed string with question mark and caret characters is escaped correctly');
assert.sameValue(RegExp.escape('.a{2}'), '\\.a\\{2\\}', 'mixed string with curly braces is escaped correctly');
assert.sameValue(RegExp.escape('.a|b'), '\\.a\\|b', 'mixed string with pipe character is escaped correctly');
assert.sameValue(RegExp.escape('.a\\b'), '\\.a\\\\b', 'mixed string with backslash is escaped correctly');
assert.sameValue(RegExp.escape('.a\\\\b'), '\\.a\\\\\\\\b', 'mixed string with backslash is escaped correctly');
assert.sameValue(RegExp.escape('.a^b'), '\\.a\\^b', 'mixed string with caret character is escaped correctly');
assert.sameValue(RegExp.escape('.a$b'), '\\.a\\$b', 'mixed string with dollar sign is escaped correctly');
assert.sameValue(RegExp.escape('.a[b]'), '\\.a\\[b\\]', 'mixed string with square brackets is escaped correctly');
assert.sameValue(RegExp.escape('.a.b(c)'), '\\.a\\.b\\(c\\)', 'mixed string with dot and parentheses is escaped correctly');
assert.sameValue(RegExp.escape('.a*b+c?d^e$f|g{2}h[i]j\\k'), '\\.a\\*b\\+c\\?d\\^e\\$f\\|g\\{2\\}h\\[i\\]j\\\\k', 'complex string with multiple special characters is escaped correctly');

assert.sameValue(
  RegExp.escape('^$\\.*+?()[]{}|'),
  '\\^\\$\\\\\\.\\*\\+\\?\\(\\)\\[\\]\\{\\}\\|',
  'Syntax characters are correctly escaped'
);
