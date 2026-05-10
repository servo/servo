// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: |
    The 'asyncTest' helper when called with async flag always returns undefined.
flags: [async]
includes: [asyncHelpers.js]
---*/
var realDone = $DONE;
var doneCalls = 0;
globalThis.$DONE = function () {
  doneCalls++;
};

(async function () {
  assert.sameValue(undefined, asyncTest({}));
  assert.sameValue(
    undefined,
    asyncTest(function () {
      return "non-thenable";
    })
  );
  assert.sameValue(
    undefined,
    asyncTest(function () {
      return Promise.resolve(true);
    })
  );
  assert.sameValue(
    undefined,
    asyncTest(function () {
      return Promise.reject(new Test262Error("oh no"));
    })
  );
})()
  .then(() => {
    assert.sameValue(doneCalls, 4, "asyncTest must call $DONE");
  })
  .then(realDone, realDone);
