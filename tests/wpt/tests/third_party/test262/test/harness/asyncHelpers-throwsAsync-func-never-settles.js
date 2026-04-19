// Copyright (C) 2024 Julián Espina. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: |
    assert.throwsAsync returns a promise that never settles if func returns a thenable that never settles.
flags: [async]
includes: [asyncHelpers.js]
---*/

var realDone = $DONE;
var doneCalls = 0
globalThis.$DONE = function () {
  doneCalls++;
}

function delay() {
  var later = Promise.resolve();
  for (var i = 0; i < 100; i++) {
    later = later.then();
  }
  return later;
}

(async function () {
  // Spy on the promise returned by an invocation of assert.throwsAsync
  // with a function that returns a thenable which never settles.
  var neverSettlingThenable = { then: function () { } };
  const p = assert.throwsAsync(TypeError, function () { return neverSettlingThenable });
  assert(p instanceof Promise, "assert.throwsAsync should return a promise");
  p.then($DONE, $DONE);
})()
  // Give it a long time to try.
  .then(delay, delay)
  .then(function () {
    assert.sameValue(doneCalls, 0, "$DONE should not have been called")
  })
  .then(realDone, realDone);
