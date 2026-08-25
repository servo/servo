// Copyright (C) 2025 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-iterator.zipkeyed
description: >
  Perform [[OwnPropertyKeys]] on the "iterables" argument after reading all properties.
info: |
  Iterator.zipKeyed ( iterables [ , options ] )
    ...
    3. Let mode be ? Get(options, "mode").
    ...
    7. If mode is "longest", then
      a. Set paddingOption to ? Get(options, "padding").
    ...
    10. Let allKeys be ? iterables.[[OwnPropertyKeys]]().
    ...
includes: [proxyTrapsHelper.js, compareArray.js]
features: [joint-iteration]
---*/

var log = [];

var iterables = new Proxy({}, allowProxyTraps({
  ownKeys(target) {
    log.push("own-keys");
    return Reflect.ownKeys(target);
  },
}));


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

Iterator.zipKeyed(iterables, options);

assert.compareArray(log, [
  "get mode",
  "get padding",
  "own-keys",
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

  Iterator.zipKeyed(iterables, options);

  assert.compareArray(log, [
    "get mode",
    "own-keys",
  ]);
}
