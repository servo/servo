// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: |
    assert.throwsAsync calls $DONE with a rejecting value if func is not a function returning a thenable.
flags: [async]
includes: [asyncHelpers.js]
---*/

async function checkRejects(func) {
  var caught = false;
  const p = assert.throwsAsync(Test262Error, func);
  assert(p instanceof Promise, "assert.throwsAsync should return a promise");
  try {
    await p;
  } catch (e) {
    caught = true;
    assert.sameValue(
      e.constructor,
      Test262Error,
      "throwsAsync should reject improper function with a Test262Error"
    );
  } finally {
    assert(
      caught,
      "assert.throwsAsync did not reject improper function " + func
    );
  }
}

asyncTest(async function () {
  await checkRejects(null);
  await checkRejects({});
  await checkRejects("string");
  await checkRejects(10);
  await checkRejects();
  await checkRejects({
    then: function (res, rej) {
      res(true);
    },
  });
  await checkRejects(function () {
    return null;
  });
  await checkRejects(function () {
    return {};
  });
  await checkRejects(function () {
    return "string";
  });
  await checkRejects(function () {
    return 10;
  });
  await checkRejects(function () {});
  await checkRejects(function () {
    return { then: null };
  });
  await checkRejects(function () {
    return { then: {} };
  });
  await checkRejects(function () {
    return { then: "string" };
  });
  await checkRejects(function () {
    return { then: 10 };
  });
  await checkRejects(function () {
    return { then: undefined };
  });
});
