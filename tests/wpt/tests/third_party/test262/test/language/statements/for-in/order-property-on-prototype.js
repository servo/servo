// Copyright 2019 Kevin Gibbons. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-enumerate-object-properties
description: Properties on the prototype are enumerated after properties on the object
features: [for-in-order]
includes: [compareArray.js]
---*/

var proto = {
  p4: 'p4',
};

var o = {
  p1: 'p1',
  p2: 'p2',
  p3: 'p3',
};

Object.setPrototypeOf(o, proto);

var keys = [];
for (var key in o) {
  keys.push(key);
}

assert.compareArray(keys, ['p1', 'p2', 'p3', 'p4']);
