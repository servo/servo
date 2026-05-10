// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: |
    The 'asyncTest' helper calls $DONE with undefined, regardless of what value the promise resolves with
flags: [async]
includes: [asyncHelpers.js]
---*/
var doneCalls = 0;
var realDone = $DONE;
globalThis.$DONE = function (noError) {
  doneCalls++;
  assert.sameValue(
    noError,
    undefined,
    "asyncTest should discard promise's resolved value"
  );
};

(async function () {
  asyncTest(function () {
    return Promise.resolve(null);
  });
})()
  .then(() => {
    assert.sameValue(doneCalls, 1, "asyncTest called $DONE with undefined");
    asyncTest(function () {
      return Promise.resolve({});
    });
  })
  .then(() => {
    assert.sameValue(doneCalls, 2, "asyncTest called $DONE with undefined");
    asyncTest(function () {
      return Promise.resolve("hi");
    });
  })
  .then(() => {
    assert.sameValue(doneCalls, 3, "asyncTest called $DONE with undefined");
    asyncTest(function () {
      return Promise.resolve(10);
    });
  })
  .then(() => {
    assert.sameValue(doneCalls, 4, "asyncTest called $DONE with undefined");
    asyncTest(function () {
      return {
        then(res, rej) {
          res(true);
        },
      };
    });
  })
  .then(() => {
    assert.sameValue(doneCalls, 5, "asyncTest called $DONE with undefined");
  })
  .then(realDone, realDone);
