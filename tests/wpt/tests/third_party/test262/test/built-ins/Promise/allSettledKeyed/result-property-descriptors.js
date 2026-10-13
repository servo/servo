// Copyright (C) 2026 Danial Asaria (Bloomberg LP). All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-createkeyedpromisecombinatorresultobject
description: >
  Promise.allSettledKeyed result and settled entry properties are data properties
info: |
  CreateKeyedPromiseCombinatorResultObject ( entries )

  ...
  1. Let obj be OrdinaryObjectCreate(null).
  2. For each Record { [[Key]], [[Value]] } entry of entries, do
    a. Perform ! CreateDataPropertyOrThrow(obj, entry.[[Key]], entry.[[Value]]).
  3. Return obj.

  The onFulfilled closure for all-settled:
    ...
    Perform ! CreateDataPropertyOrThrow(obj, "status", "fulfilled").
    Perform ! CreateDataPropertyOrThrow(obj, "value", value).

  The onRejected closure:
    ...
    Perform ! CreateDataPropertyOrThrow(obj, "status", "rejected").
    Perform ! CreateDataPropertyOrThrow(obj, "reason", error).
includes: [asyncHelpers.js, propertyHelper.js]
flags: [async]
features: [await-dictionary]
---*/

var error = new Test262Error();

asyncTest(function() {
  return Promise.allSettledKeyed({
    fulfilled: Promise.resolve(1),
    rejected: Promise.reject(error)
  }).then(function(result) {
    assert.sameValue(Object.getPrototypeOf(result), null, "result is null-prototype");
    assert.sameValue(Object.getPrototypeOf(result.fulfilled), Object.prototype, "fulfilled entry prototype");
    assert.sameValue(Object.getPrototypeOf(result.rejected), Object.prototype, "rejected entry prototype");

    // Capture the properties eagerly because `verifyProperty` will delete them
    // as part of the `configurable` check.
    var fulfilledEntry = result.fulfilled;
    var rejectedEntry = result.rejected;

    verifyProperty(result, "fulfilled", {
      value: fulfilledEntry,
      writable: true,
      enumerable: true,
      configurable: true
    });
    verifyProperty(result, "rejected", {
      value: rejectedEntry,
      writable: true,
      enumerable: true,
      configurable: true
    });

    verifyProperty(fulfilledEntry, "status", {
      value: "fulfilled",
      writable: true,
      enumerable: true,
      configurable: true
    });
    verifyProperty(fulfilledEntry, "value", {
      value: 1,
      writable: true,
      enumerable: true,
      configurable: true
    });
    verifyProperty(rejectedEntry, "status", {
      value: "rejected",
      writable: true,
      enumerable: true,
      configurable: true
    });
    verifyProperty(rejectedEntry, "reason", {
      value: error,
      writable: true,
      enumerable: true,
      configurable: true
    });
  });
});
