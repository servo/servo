// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    ToObject conversion from Number: create a new Number object
    whose [[value]] property is set to the value of the number
es5id: 9.9_A4
description: Converting from various numbers to Object
---*/
assert.sameValue(Object(0).valueOf(), 0, 'Object(0).valueOf() must return 0');
assert.sameValue(typeof Object(0), "object", 'The value of `typeof Object(0)` is expected to be "object"');

assert.sameValue(
  Object(0).constructor.prototype,
  Number.prototype,
  'The value of Object(0).constructor.prototype is expected to equal the value of Number.prototype'
);

assert.sameValue(Object(-0).valueOf(), -0, 'Object(-0).valueOf() must return -0');
assert.sameValue(typeof Object(-0), "object", 'The value of `typeof Object(-0)` is expected to be "object"');

assert.sameValue(
  Object(-0).constructor.prototype,
  Number.prototype,
  'The value of Object(-0).constructor.prototype is expected to equal the value of Number.prototype'
);

assert.sameValue(Object(1).valueOf(), 1, 'Object(1).valueOf() must return 1');
assert.sameValue(typeof Object(1), "object", 'The value of `typeof Object(1)` is expected to be "object"');

assert.sameValue(
  Object(1).constructor.prototype,
  Number.prototype,
  'The value of Object(1).constructor.prototype is expected to equal the value of Number.prototype'
);

assert.sameValue(Object(-1).valueOf(), -1, 'Object(-1).valueOf() must return -1');
assert.sameValue(typeof Object(-1), "object", 'The value of `typeof Object(-1)` is expected to be "object"');

assert.sameValue(
  Object(-1).constructor.prototype,
  Number.prototype,
  'The value of Object(-1).constructor.prototype is expected to equal the value of Number.prototype'
);

assert.sameValue(
  Object(Number.MIN_VALUE).valueOf(),
  Number.MIN_VALUE,
  'Object(Number.MIN_VALUE).valueOf() returns Number.MIN_VALUE'
);

assert.sameValue(
  typeof Object(Number.MIN_VALUE),
  "object",
  'The value of `typeof Object(Number.MIN_VALUE)` is expected to be "object"'
);

assert.sameValue(
  Object(Number.MIN_VALUE).constructor.prototype,
  Number.prototype,
  'The value of Object(Number.MIN_VALUE).constructor.prototype is expected to equal the value of Number.prototype'
);

assert.sameValue(
  Object(Number.MAX_VALUE).valueOf(),
  Number.MAX_VALUE,
  'Object(Number.MAX_VALUE).valueOf() returns Number.MAX_VALUE'
);

assert.sameValue(
  typeof Object(Number.MAX_VALUE),
  "object",
  'The value of `typeof Object(Number.MAX_VALUE)` is expected to be "object"'
);

assert.sameValue(
  Object(Number.MAX_VALUE).constructor.prototype,
  Number.prototype,
  'The value of Object(Number.MAX_VALUE).constructor.prototype is expected to equal the value of Number.prototype'
);

assert.sameValue(
  Object(Number.POSITIVE_INFINITY).valueOf(),
  Number.POSITIVE_INFINITY,
  'Object(Number.POSITIVE_INFINITY).valueOf() returns Number.POSITIVE_INFINITY'
);

assert.sameValue(
  typeof Object(Number.POSITIVE_INFINITY),
  "object",
  'The value of `typeof Object(Number.POSITIVE_INFINITY)` is expected to be "object"'
);

assert.sameValue(
  Object(Number.POSITIVE_INFINITY).constructor.prototype,
  Number.prototype,
  'The value of Object(Number.POSITIVE_INFINITY).constructor.prototype is expected to equal the value of Number.prototype'
);

assert.sameValue(
  Object(Number.NEGATIVE_INFINITY).valueOf(),
  Number.NEGATIVE_INFINITY,
  'Object(Number.NEGATIVE_INFINITY).valueOf() returns Number.NEGATIVE_INFINITY'
);

assert.sameValue(
  typeof Object(Number.NEGATIVE_INFINITY),
  "object",
  'The value of `typeof Object(Number.NEGATIVE_INFINITY)` is expected to be "object"'
);

assert.sameValue(
  Object(Number.NEGATIVE_INFINITY).constructor.prototype,
  Number.prototype,
  'The value of Object(Number.NEGATIVE_INFINITY).constructor.prototype is expected to equal the value of Number.prototype'
);

assert.sameValue(Object(NaN).valueOf(), NaN, 'Object(NaN).valueOf() returns NaN');

assert.sameValue(
  typeof Object(Number.NaN),
  "object",
  'The value of `typeof Object(Number.NaN)` is expected to be "object"'
);

assert.sameValue(
  Object(Number.NaN).constructor.prototype,
  Number.prototype,
  'The value of Object(Number.NaN).constructor.prototype is expected to equal the value of Number.prototype'
);

assert.sameValue(Object(1.2345).valueOf(), 1.2345, 'Object(1.2345).valueOf() must return 1.2345');
assert.sameValue(typeof Object(1.2345), "object", 'The value of `typeof Object(1.2345)` is expected to be "object"');

assert.sameValue(
  Object(1.2345).constructor.prototype,
  Number.prototype,
  'The value of Object(1.2345).constructor.prototype is expected to equal the value of Number.prototype'
);

assert.sameValue(Object(-1.2345).valueOf(), -1.2345, 'Object(-1.2345).valueOf() must return -1.2345');
assert.sameValue(typeof Object(-1.2345), "object", 'The value of `typeof Object(-1.2345)` is expected to be "object"');

assert.sameValue(
  Object(-1.2345).constructor.prototype,
  Number.prototype,
  'The value of Object(-1.2345).constructor.prototype is expected to equal the value of Number.prototype'
);
