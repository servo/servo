// Copyright (C) 2025 J. S. Choi. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.fromasync
description: >
  Array.fromAsync closes any sync-iterable input with a rejecting thenable.
includes: [asyncHelpers.js]
flags: [async]
features: [Array.fromAsync]
---*/

asyncTest(async function () {
  const expectedValue = {};
  let finallyCounter = 0;
  const inputThenable = {
    then(resolve, reject) {
      reject();
    },
  };
  function* createInput() {
    try {
      yield inputThenable;
    } finally {
      finallyCounter++;
    }
  }
  const input = createInput();
  try {
    await Array.fromAsync(input);
  } finally {
    assert.sameValue(finallyCounter, 1);
  }
});
