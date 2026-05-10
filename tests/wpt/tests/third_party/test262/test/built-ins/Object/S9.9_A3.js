// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    ToObject conversion from Boolean: create a new Boolean object
    whose [[value]] property is set to the value of the boolean
es5id: 9.9_A3
description: Trying to convert from Boolean to Object
---*/
assert.sameValue(Object(true).valueOf(), true, 'Object(true).valueOf() must return true');
assert.sameValue(typeof Object(true), "object", 'The value of `typeof Object(true)` is expected to be "object"');

assert.sameValue(
  Object(true).constructor.prototype,
  Boolean.prototype,
  'The value of Object(true).constructor.prototype is expected to equal the value of Boolean.prototype'
);

assert.sameValue(Object(false).valueOf(), false, 'Object(false).valueOf() must return false');
assert.sameValue(typeof Object(false), "object", 'The value of `typeof Object(false)` is expected to be "object"');

assert.sameValue(
  Object(false).constructor.prototype,
  Boolean.prototype,
  'The value of Object(false).constructor.prototype is expected to equal the value of Boolean.prototype'
);
