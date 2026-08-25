// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-delete-operator-runtime-semantics-evaluation
description: >
    TypeError isn't thrown when deleting configurable accessor property
---*/

var obj = {};
Object.defineProperty(obj, 'prop', {
  get: function() {
    return 'abc';
  },
  configurable: true,
});

delete obj.prop;

assert.sameValue(
  obj.hasOwnProperty('prop'),
  false,
  'obj.hasOwnProperty("prop")'
);
