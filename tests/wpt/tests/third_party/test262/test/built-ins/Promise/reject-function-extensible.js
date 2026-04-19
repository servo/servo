// Copyright (C) 2015 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es6id: 25.4.1.3.1
description: The [[Extensible]] slot of Promise Reject functions
info: |
  17 ECMAScript Standard Built-in Objects:
    Unless specified otherwise, the [[Extensible]] internal slot
    of a built-in object initially has the value true.
---*/

var rejectFunction;
new Promise(function(resolve, reject) {
  rejectFunction = reject;
});

assert(Object.isExtensible(rejectFunction));
