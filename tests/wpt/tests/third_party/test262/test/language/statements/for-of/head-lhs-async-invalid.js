// Copyright (C) 2021 Kevin Gibbons. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: leading `async` token in for-of LHS
info: |
  The `async` token is disallowed in the LHS when followed by `of`
esid: sec-for-in-and-for-of-statements
negative:
  phase: parse
  type: SyntaxError
---*/

$DONOTEVALUATE();

var async;
for (async of [1]) ;
