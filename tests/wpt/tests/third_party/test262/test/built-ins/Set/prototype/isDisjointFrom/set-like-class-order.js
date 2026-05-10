// Copyright (C) 2023 Anthony Frehner and Kevin Gibbons. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-set.prototype.isdisjointfrom
description: Set.prototype.isDisjointFrom calls a Set-like class's methods in order
features: [set-methods]
includes: [compareArray.js]
---*/

let observedOrder = [];

function observableIterator() {
  let values = ["a", "b", "c"];
  let index = 0;
  return {
    get next() {
      observedOrder.push("getting next");
      return function () {
        observedOrder.push("calling next");
        return {
          get done() {
            observedOrder.push("getting done");
            return index >= values.length;
          },
          get value() {
            observedOrder.push("getting value");
            return values[index++];
          },
        };
      };
    },
  };
}

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
      observedOrder.push("calling keys");
      return observableIterator();
    };
  }
}

// this is smaller than argument
{
  observedOrder = [];

  const s1 = new Set(["x", "b"]);
  const s2 = new MySetLike();
  const result = s1.isDisjointFrom(s2);

  const expectedOrder = [
    "getting size",
    "ToNumber(size)",
    "getting has",
    "getting keys",
    // two calls to has
    "calling has",
    "calling has",
  ];

  assert.sameValue(result, false);
  assert.compareArray(observedOrder, expectedOrder);
}

// this is same size as argument - stops eagerly
{
  observedOrder = [];

  const s1 = new Set(["x", "b", "y"]);
  const s2 = new MySetLike();
  const result = s1.isDisjointFrom(s2);

  const expectedOrder = [
    "getting size",
    "ToNumber(size)",
    "getting has",
    "getting keys",
    // two calls to has
    "calling has",
    "calling has",
  ];

  assert.sameValue(result, false);
  assert.compareArray(observedOrder, expectedOrder);
}

// this is same size as argument - full run
{
  observedOrder = [];

  const s1 = new Set(["x", "y", "z"]);
  const s2 = new MySetLike();
  const result = s1.isDisjointFrom(s2);

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

// this is larger than argument - stops eagerly
{
  observedOrder = [];

  const s1 = new Set(["x", "b", "y", "z"]);
  const s2 = new MySetLike();
  const result = s1.isDisjointFrom(s2);

  const expectedOrder = [
    "getting size",
    "ToNumber(size)",
    "getting has",
    "getting keys",
    "calling keys",
    "getting next",
    // first iteration, has value
    "calling next",
    "getting done",
    "getting value",
    // second iteration, has value
    "calling next",
    "getting done",
    "getting value",
  ];

  assert.sameValue(result, false);
  assert.compareArray(observedOrder, expectedOrder);
}

// this is larger than argument - full run
{
  observedOrder = [];

  const s1 = new Set(["x", "y", "z", "w"]);
  const s2 = new MySetLike();
  const result = s1.isDisjointFrom(s2);

  const expectedOrder = [
    "getting size",
    "ToNumber(size)",
    "getting has",
    "getting keys",
    "calling keys",
    "getting next",
    // first iteration, has value
    "calling next",
    "getting done",
    "getting value",
    // second iteration, has value
    "calling next",
    "getting done",
    "getting value",
    // third iteration, has value
    "calling next",
    "getting done",
    "getting value",
    // fourth iteration, done
    "calling next",
    "getting done",
  ];

  assert.sameValue(result, true);
  assert.compareArray(observedOrder, expectedOrder);
}
