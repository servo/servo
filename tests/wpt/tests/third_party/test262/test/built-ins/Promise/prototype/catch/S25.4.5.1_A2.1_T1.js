// Copyright 2014 Cubane Canada, Inc.  All rights reserved.
// See LICENSE for details.

/*---
info: |
    catch is a method on a Promise
es6id: S25.4.5.1_A2.1_T1
author: Sam Mikes
description: catch is a method on a Promise
---*/

var p = Promise.resolve(3);

assert(
  !!(p.catch instanceof Function),
  'The value of !!(p.catch instanceof Function) is expected to be true'
);
