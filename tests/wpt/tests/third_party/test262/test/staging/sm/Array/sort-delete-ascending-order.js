// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
includes: [compareArray.js]
description: |
  pending
esid: pending
---*/
// Calls Array.prototype.sort and tests that properties are deleted in the same order in the
// native and the self-hosted implementation.

function createProxy() {
  var deleted = [];
  var proxy = new Proxy([, , 0], {
    deleteProperty(t, pk){
      deleted.push(pk);
      return delete t[pk];
    }
  });

  return {proxy, deleted};
}

function compareFn(a, b) {
  return a < b ? -1 : a > b ? 1 : 0;
}

// Sort an array without a comparator function. This calls the native sort implementation.

var {proxy, deleted} = createProxy();

assert.compareArray(deleted, []);
proxy.sort()
assert.compareArray(deleted, ["1", "2"]);

// Now sort an array with a comparator function. This calls the self-hosted sort implementation.

var {proxy, deleted} = createProxy();

assert.compareArray(deleted, []);
proxy.sort(compareFn);
assert.compareArray(deleted, ["1", "2"]);

