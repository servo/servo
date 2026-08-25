// Copyright (C) 2023 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.fromasync
description: >
  Order of user-observable operations on a custom this-value and its instances
includes: [compareArray.js, asyncHelpers.js]
flags: [async]
features: [Array.fromAsync]
---*/

function formatPropertyName(propertyKey, objectName = "") {
  switch (typeof propertyKey) {
    case "symbol":
      if (Symbol.keyFor(propertyKey) !== undefined) {
        return `${objectName}[Symbol.for('${Symbol.keyFor(propertyKey)}')]`;
      } else if (propertyKey.description.startsWith('Symbol.')) {
        return `${objectName}[${propertyKey.description}]`;
      } else {
        return `${objectName}[Symbol('${propertyKey.description}')]`
      }
    case "string":
      if (propertyKey !== String(Number(propertyKey)))
        return objectName ? `${objectName}.${propertyKey}` : propertyKey;
      // fall through
    default:
      // integer or string integer-index
      return `${objectName}[${propertyKey}]`;
  }
}

asyncTest(async function () {
  const expectedCalls = [
    "construct MyArray",
    "defineProperty A[0]",
    "defineProperty A[1]",
    "set A.length"
  ];
  const actualCalls = [];

  function MyArray(...args) {
    actualCalls.push("construct MyArray");
    return new Proxy(Object.create(null), {
      set(target, key, value) {
        actualCalls.push(`set ${formatPropertyName(key, "A")}`);
        return Reflect.set(target, key, value);
      },
      defineProperty(target, key, descriptor) {
        actualCalls.push(`defineProperty ${formatPropertyName(key, "A")}`);
        return Reflect.defineProperty(target, key, descriptor);
      }
    });
  }

  let result = await Array.fromAsync.call(MyArray, [1, 2]);
  assert.compareArray(expectedCalls, actualCalls, "order of operations for array argument");

  actualCalls.splice(0);  // reset

  const expectedCallsForArrayLike = [
    "construct MyArray",
    "defineProperty A[0]",
    "defineProperty A[1]",
    "set A.length"
  ];
  result = await Array.fromAsync.call(MyArray, {
    length: 2,
    0: 1,
    1: 2
  });
  assert.compareArray(expectedCallsForArrayLike, actualCalls, "order of operations for array-like argument");
});
