// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: |
    The 'asyncTest' helper calls $DONE with the rejection value if the test function rejects.
flags: [async]
includes: [asyncHelpers.js, compareArray.js]
---*/
const rejectionValues = [];
var realDone = $DONE;
globalThis.$DONE = function (mustBeDefined) {
  rejectionValues.push(mustBeDefined);
};
const someObject = {};

(async function () {
  asyncTest(function () {
    return Promise.reject(null);
  });
})()
  .then(() => {
    asyncTest(function () {
      return Promise.reject(someObject);
    });
  })
  .then(() => {
    asyncTest(function () {
      return Promise.reject("hi");
    });
  })
  .then(() => {
    asyncTest(function () {
      return Promise.reject(10);
    });
  })
  .then(() => {
    asyncTest(function () {
      return {
        then(res, rej) {
          rej(true);
        },
      };
    });
  })
  .then(() => {
    assert.compareArray(rejectionValues, [null, someObject, "hi", 10, true]);
  })
  .then(realDone, realDone);
