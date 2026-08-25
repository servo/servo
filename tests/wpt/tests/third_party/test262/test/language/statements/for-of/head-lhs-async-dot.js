// Copyright (C) 2021 Kevin Gibbons. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: leading `async` token in for-of LHS
info: |
  The `async` token is allowed in the LHS if not followed by `of`
esid: sec-for-in-and-for-of-statements
---*/

var async = { x: 0 };

for (async.x of [1]) ;

assert.sameValue(async.x, 1);
