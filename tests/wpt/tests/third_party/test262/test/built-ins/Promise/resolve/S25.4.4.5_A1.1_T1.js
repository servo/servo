// Copyright 2014 Cubane Canada, Inc.  All rights reserved.
// See LICENSE for details.

/*---
info: |
   Promise.resolve
es6id: S25.4.4.5_A1.1_T1
author: Sam Mikes
description: Promise.resolve is a function
---*/
assert.sameValue(
  typeof Promise.resolve,
  "function",
  'The value of `typeof Promise.resolve` is expected to be "function"'
);
