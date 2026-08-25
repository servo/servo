// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-delete-operator-runtime-semantics-evaluation
description: >
    Strict Mode - TypeError is thrown when deleting non-configurable
    data property
flags: [onlyStrict]
---*/

var obj = {};
Object.defineProperty(obj, 'prop', {
  value: 'abc',
  configurable: false,
});
assert.throws(TypeError, function() {
  delete obj.prop;
});
assert.sameValue(obj.prop, 'abc', 'obj.prop');
