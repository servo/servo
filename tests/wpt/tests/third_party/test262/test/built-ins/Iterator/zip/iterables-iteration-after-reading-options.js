// Copyright (C) 2025 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-iterator.zip
description: >
  Perform iteration of the "iterables" argument after reading all properties.
info: |
  Iterator.zip ( iterables [ , options ] )
    ...
    3. Let mode be ? Get(options, "mode").
    ...
    7. If mode is "longest", then
      a. Set paddingOption to ? Get(options, "padding").
    ...
    10. Let inputIter be ? GetIterator(iterables, sync).
    ...
includes: [compareArray.js]
features: [joint-iteration]
---*/

var log = [];

var iterables = {
  [Symbol.iterator]() {
    log.push("get iterator");
    return this;
  },
  next() {
    return {done: true};
  }
};

var options = {
  get mode() {
    log.push("get mode");
    return "longest";
  },
  get padding() {
    log.push("get padding");
    return [];
  }
};

Iterator.zip(iterables, options);

assert.compareArray(log, [
  "get mode",
  "get padding",
  "get iterator",
]);

for (var mode of [undefined, "shortest", "strict"]) {
  log.length = 0;

  options = {
    get mode() {
      log.push("get mode");
      return mode;
    },
    get padding() {
      log.push("unexpected get padding");
      return [];
    }
  };

  Iterator.zip(iterables, options);

  assert.compareArray(log, [
    "get mode",
    "get iterator",
  ]);
}
