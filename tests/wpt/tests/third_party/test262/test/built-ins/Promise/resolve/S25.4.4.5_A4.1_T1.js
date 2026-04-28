// Copyright 2014 Cubane Canada, Inc.  All rights reserved.
// See LICENSE for details.

/*---
info: |
    Section 25.4.1.4.2
es6id: S25.4.4.5_A3.1_T1
author: Sam Mikes
description: self-resolved Promise throws TypeError
flags: [async]
---*/

var resolveP,
  p = new Promise(function(resolve) {
    resolveP = resolve;
  });

resolveP(p);

p.then(function() {
  throw new Test262Error("Should not fulfill: should reject with TypeError.");
}, function(err) {
  assert(!!(err instanceof TypeError), 'The value of !!(err instanceof TypeError) is expected to be true');
}).then($DONE, $DONE);
