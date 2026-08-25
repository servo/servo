// Copyright (C) 2025 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-iterator.zipkeyed
description: >
  Perform keys iteration on the "padding" option.
info: |
  Iterator.zipKeyed ( iterables [ , options ] )
    ...
    14. If mode is "longest", then
      ...
      b. Else,
        i. For each element key of keys, do
          1. Let value be Completion(Get(paddingOption, key)).
          ...
includes: [proxyTrapsHelper.js, compareArray.js]
features: [joint-iteration]
---*/

function makeKeys(k) {
  var str = "abcdefgh";
  assert(k <= str.length, "more than eight keys are unsupported");
  return str.slice(0, k).split("");
}

function fromKeys(keys, value) {
  return Object.fromEntries(keys.map(function(k) {
    return [k, value];
  }));
}

for (var n = 0; n <= 5; ++n) {
  // Create an object with |n| properties.
  var keys = makeKeys(n);
  var iterables = fromKeys(keys, []);

  for (var k = 0; k <= n + 2; ++k) {
    var log = [];

    // Create a padding object with |k| properties. Ensure only [[Get]] is
    // called.
    var padding = new Proxy(fromKeys(makeKeys(k), undefined), allowProxyTraps({
      get(target, propertyKey, receiver) {
        log.push(propertyKey);
        return Reflect.get(target, propertyKey, receiver);
      },
    }));

    Iterator.zipKeyed(iterables, {mode: "longest", padding});

    // [[Get]] happened for all keys from |keys|.
    assert.compareArray(log, keys);
  }
}
