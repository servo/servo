// Copyright 2019 Kevin Gibbons. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-enumerate-object-properties
description: Enumerable properties the prototype which are shadowed by non-enumerable properties on the object are not enumerated
features: [for-in-order]
includes: [compareArray.js]
---*/

var proto = {
  p2: 'p2',
};

var o = Object.create(proto, {
  'p1': {
    value: 'p1',
    enumerable: true,
  },
  'p2': {
    value: 'p1',
    enumerable: false,
  },
});



var keys = [];
for (var key in o) {
  keys.push(key);
}

assert.compareArray(keys, ['p1']);
