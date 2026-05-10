// Copyright 2014 Cubane Canada, Inc.  All rights reserved.
// See LICENSE for details.

/*---
info: Promise.all is resolved with a new empty array
es6id: 25.4.4.1_A2.3_T2
author: Sam Mikes
description: Promise.all([]) returns a Promise for an empty array
flags: [async]
---*/

var arg = [];

Promise.all(arg).then(function(result) {
  assert.sameValue(result.length, 0, 'The value of result.length is expected to be 0');
}).then($DONE, $DONE);
