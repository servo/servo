// Copyright 2014 Cubane Canada, Inc.  All rights reserved.
// See LICENSE for details.

/*---
info: |
   Promise.resolve
es6id: S25.4.4.5
author: Sam Mikes
description: Promise.resolve delegates to foreign thenable
includes: [promiseHelper.js]
flags: [async]
---*/

var sequence = [];

var thenable = {
  then: function(onResolve, onReject) {

    sequence.push(3);
    assert.sameValue(sequence.length, 3);
    checkSequence(sequence, "thenable.then called");

    assert.sameValue(this, thenable, "thenable.then called with `thenable` as `this`");

    return onResolve('resolved');
  }
};

sequence.push(1);
assert.sameValue(sequence.length, 1);
checkSequence(sequence, "no async calls yet");

var p = Promise.resolve(thenable);

sequence.push(2);
assert.sameValue(sequence.length, 2);
checkSequence(sequence, "thenable.then queued but not yet called");

p.then(function(r) {
  sequence.push(4);
  assert.sameValue(sequence.length, 4);
  checkSequence(sequence, "all done");

  assert.sameValue(r, 'resolved');
}).then($DONE, $DONE);
