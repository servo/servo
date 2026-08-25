// Copyright 2014 Cubane Canada, Inc.  All rights reserved.
// See LICENSE for details.

/*---
info: Promise.all([]) returns a promise for a new empty array
es6id: 25.4.4.1_A2.3_T1
author: Sam Mikes
description: Promise.all([]) returns a promise for an array
flags: [async]
---*/

var arg = [];

Promise.all(arg).then(function(result) {
  assert(!!(result instanceof Array), 'The value of !!(result instanceof Array) is expected to be true');
}).then($DONE, $DONE);
