// Copyright (C) 2024 Leo Balter. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-regexp.escape
description: Escaped characters (simple assertions)
info: |
  RegExp.escape ( string )

  This method produces a new string in which certain characters have been escaped.
  These characters are: . * + ? ^ $ | ( ) [ ] { } \

features: [RegExp.escape]
---*/

assert.sameValue(RegExp.escape('.'), '\\.', 'dot character is escaped correctly');
assert.sameValue(RegExp.escape('*'), '\\*', 'asterisk character is escaped correctly');
assert.sameValue(RegExp.escape('+'), '\\+', 'plus character is escaped correctly');
assert.sameValue(RegExp.escape('?'), '\\?', 'question mark character is escaped correctly');
assert.sameValue(RegExp.escape('^'), '\\^', 'caret character is escaped correctly');
assert.sameValue(RegExp.escape('$'), '\\$', 'dollar character is escaped correctly');
assert.sameValue(RegExp.escape('|'), '\\|', 'pipe character is escaped correctly');
assert.sameValue(RegExp.escape('('), '\\(', 'open parenthesis character is escaped correctly');
assert.sameValue(RegExp.escape(')'), '\\)', 'close parenthesis character is escaped correctly');
assert.sameValue(RegExp.escape('['), '\\[', 'open bracket character is escaped correctly');
assert.sameValue(RegExp.escape(']'), '\\]', 'close bracket character is escaped correctly');
assert.sameValue(RegExp.escape('{'), '\\{', 'open brace character is escaped correctly');
assert.sameValue(RegExp.escape('}'), '\\}', 'close brace character is escaped correctly');
assert.sameValue(RegExp.escape('\\'), '\\\\', 'backslash character is escaped correctly');
