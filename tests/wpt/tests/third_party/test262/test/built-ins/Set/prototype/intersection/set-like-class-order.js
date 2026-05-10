// Copyright (C) 2023 Anthony Frehner and Kevin Gibbons. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-set.prototype.intersection
description: Set.prototype.intersection calls a Set-like class's methods in order
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

  const s1 = new Set(["a", "d"]);
  const s2 = new MySetLike();
  const combined = s1.intersection(s2);

  const expectedOrder = [
    "getting size",
    "ToNumber(size)",
    "getting has",
    "getting keys",
    // two calls to has
    "calling has",
    "calling has",
  ];

  assert.compareArray([...combined], ["a"]);
  assert.compareArray(observedOrder, expectedOrder);
}

// this is same size as argument
{
  observedOrder = [];

  const s1 = new Set(["a", "b", "d"]);
  const s2 = new MySetLike();
  const combined = s1.intersection(s2);

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

  assert.compareArray([...combined], ["a", "b"]);
  assert.compareArray(observedOrder, expectedOrder);
}

// this is larger than argument
{
  observedOrder = [];

  const s1 = new Set(["a", "b", "c", "d"]);
  const s2 = new MySetLike();
  const combined = s1.intersection(s2);

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
    // fourth iteration, no value; ends
    "calling next",
    "getting done",
  ];

  assert.compareArray([...combined], ["a", "b", "c"]);
  assert.compareArray(observedOrder, expectedOrder);
}
