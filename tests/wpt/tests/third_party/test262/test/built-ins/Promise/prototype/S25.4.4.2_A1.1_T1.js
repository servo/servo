// Copyright 2014 Cubane Canada, Inc.  All rights reserved.
// See LICENSE for details.

/*---
info: |
    Promise prototype object exists, is object, not enumerable, writable,
    or configurable
es6id: S25.4.4.2_A1.1_T1
author: Sam Mikes
description: Promise prototype exists
---*/
assert.notSameValue(
  Promise.prototype,
  undefined,
  'The value of Promise.prototype is expected to not equal ``undefined``'
);
