// Copyright (C) 2023 Anthony Frehner and Kevin Gibbons. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-set.prototype.issubsetof
description: Set.prototype.isSubsetOf calls a Set-like class's methods in order
features: [set-methods]
includes: [compareArray.js]
---*/

let observedOrder = [];

class MySetLike {
  get size() {
    observedOrder.push("getting size");
    return {
      valueOf: function () {
        observedOrder.push("ToNumber(size)");
        return 3;
      },
    };
  }
  get has() {
    observedOrder.push("getting has");
    return function (v) {
      observedOrder.push("calling has");
      return ["a", "b", "c"].indexOf(v) !== -1;
    };
  }
  get keys() {
    observedOrder.push("getting keys");
    return function () {
      throw new Test262Error("Set.prototype.isSubsetOf should not call its argument's keys iterator");
    };
  }
}

// this is smaller than argument - stops eagerly
{
  observedOrder = [];

  const s1 = new Set(["d", "a"]);
  const s2 = new MySetLike();
  const result = s1.isSubsetOf(s2);

  const expectedOrder = [
    "getting size",
    "ToNumber(size)",
    "getting has",
    "getting keys",
    // one call to has
    "calling has",
  ];

  assert.sameValue(result, false);
  assert.compareArray(observedOrder, expectedOrder);
}

// this is smaller than argument - full run
{
  observedOrder = [];

  const s1 = new Set(["a", "b"]);
  const s2 = new MySetLike();
  const result = s1.isSubsetOf(s2);

  const expectedOrder = [
    "getting size",
    "ToNumber(size)",
    "getting has",
    "getting keys",
    // two calls to has
    "calling has",
    "calling has",
  ];

  assert.sameValue(result, true);
  assert.compareArray(observedOrder, expectedOrder);
}

// this is same size as argument
{
  observedOrder = [];

  const s1 = new Set(["a", "b", "c"]);
  const s2 = new MySetLike();
  const result = s1.isSubsetOf(s2);

  const expectedOrder = [
    "getting size",
    "ToNumber(size)",
    "getting has",
    "getting keys",
    // three calls to has
    "calling has",
    "calling has",
    "calling has",
  ];

  assert.sameValue(result, true);
  assert.compareArray(observedOrder, expectedOrder);
}

// this is larger than argument
{
  observedOrder = [];

  const s1 = new Set(["a", "b", "c", "d"]);
  const s2 = new MySetLike();
  const result = s1.isSubsetOf(s2);

  const expectedOrder = [
    "getting size",
    "ToNumber(size)",
    "getting has",
    "getting keys",
    // no calls to has
  ];

  assert.sameValue(result, false);
  assert.compareArray(observedOrder, expectedOrder);
}
