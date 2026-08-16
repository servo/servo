// Copyright (C) 2026 Danial Asaria (Bloomberg LP). All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-createkeyedpromisecombinatorresultobject
description: >
  Promise.allKeyed result properties are data properties
info: |
  CreateKeyedPromiseCombinatorResultObject ( entries )

  ...
  1. Let obj be OrdinaryObjectCreate(null).
  2. For each Record { [[Key]], [[Value]] } entry of entries, do
    a. Perform ! CreateDataPropertyOrThrow(obj, entry.[[Key]], entry.[[Value]]).
  3. Return obj.
includes: [asyncHelpers.js, propertyHelper.js]
flags: [async]
features: [await-dictionary]
---*/

asyncTest(function() {
  return Promise.allKeyed({
    first: Promise.resolve(1),
    second: Promise.resolve(2)
  }).then(function(result) {
    assert.sameValue(Object.getPrototypeOf(result), null, "result is null-prototype");
    verifyProperty(result, "first", {
      value: 1,
      writable: true,
      enumerable: true,
      configurable: true
    });
    verifyProperty(result, "second", {
      value: 2,
      writable: true,
      enumerable: true,
      configurable: true
    });
  });
});
