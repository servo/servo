// Copyright (C) 2019 Leo Balter. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-rules-of-automatic-semicolon-insertion
description: ASI at the end of a do-while statement without a new line terminator
info: |
  1. When, as the source text is parsed from left to right, a token (called the offending token) is
  encountered that is not allowed by any production of the grammar, then a semicolon is
  automatically inserted before the offending token if one or more of the following conditions is
  true:

  ...
  - The previous token is ) and the inserted semicolon would then be parsed as the terminating
    semicolon of a do-while statement (13.7.2).
---*/

var x;
do break ; while (0) x = 42;
assert.sameValue(x, 42);

x = 0;
do do do ; while (x) while (x) while (x) x = 39;
assert.sameValue(x, 39);
