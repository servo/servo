// Copyright (C) 2019 Leo Balter. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-promise.allsettled
description: Promise.allSettled([]) returns immediately
includes: [promiseHelper.js]
flags: [async]
features: [Promise.allSettled]
---*/

var sequence = [];

Promise.allSettled([]).then(function() {
  sequence.push(2);
}).catch($DONE);

Promise.resolve().then(function() {
  sequence.push(3);
}).then(function() {
  sequence.push(4);
  assert.sameValue(sequence.length, 4);
checkSequence(sequence, 'Promises resolved in unexpected sequence');
}).then($DONE, $DONE);

sequence.push(1);
