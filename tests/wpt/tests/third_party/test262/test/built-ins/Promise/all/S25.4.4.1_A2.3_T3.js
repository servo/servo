// Copyright 2014 Cubane Canada, Inc.  All rights reserved.
// See LICENSE for details.

/*---
info: Promise.all([]) is resolved with Promise for a new empty array
es6id: 25.4.4.1_A2.3_T3
author: Sam Mikes
description: Promise.all([]) is resolved with a Promise for a new array
flags: [async]
---*/

var arg = [];

Promise.all(arg).then(function(result) {
  assert.notSameValue(result, arg, 'The value of result is expected to not equal the value of `arg`');
}).then($DONE, $DONE);
