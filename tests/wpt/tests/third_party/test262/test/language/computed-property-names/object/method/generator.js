// Copyright (C) 2014 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 12.2.5
description: >
    computed property names can be used as the name of a generator method in an object
includes: [compareArray.js]
---*/
var object = {
  *['a']() {
    yield 1;
    yield 2;
  }
};
assert.compareArray(
  Object.keys(object),
  ['a']
);
