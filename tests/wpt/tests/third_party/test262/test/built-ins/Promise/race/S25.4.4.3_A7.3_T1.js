// Copyright 2014 Cubane Canada, Inc.  All rights reserved.
// See LICENSE for details.

/*---
es6id: S25.4.4.3_A7.3_T1
author: Sam Mikes
description: Promise.race([p1, p2]) settles when first settles
flags: [async]
---*/

var resolveP1, rejectP2,
  p1 = new Promise(function(resolve) {
    resolveP1 = resolve;
  }),
  p2 = new Promise(function(resolve, reject) {
    rejectP2 = reject;
  });

rejectP2(new Error("Promise.race should not see this if P1 already resolved"));
resolveP1(1);

Promise.race([p1, p2]).then(function(result) {
  assert.sameValue(result, 1, 'The value of result is expected to be 1');
}).then($DONE, $DONE);
