// Copyright 2014 Cubane Canada, Inc.  All rights reserved.
// See LICENSE for details.

/*---
es6id: S25.4.4.3_A7.3_T2
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

Promise.race([p1, p2]).then(function() {
  throw new Test262Error("Should not be fulfilled: expected rejection.");
}, function(result) {
  assert.sameValue(result, 2, 'The value of result is expected to be 2');
}).then($DONE, $DONE);

rejectP2(2);
resolveP1(1);
