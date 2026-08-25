// Copyright 2014 Cubane Canada, Inc.  All rights reserved.
// See LICENSE for details.

/*---
info: |
    Promise.prototype.constructor is the Promise constructor
es6id: S25.4.5_A3.1_T1
author: Sam Mikes
description: Promise.prototype.constructor is the Promise constructor
---*/
assert.sameValue(
  Promise.prototype.constructor,
  Promise,
  'The value of Promise.prototype.constructor is expected to equal the value of Promise'
);
