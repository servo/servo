// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    The [[Prototype]] property of the newly constructed object
    is set to the original Date prototype object, the one that is the
    initial value of Date.prototype
esid: sec-date-value
description: Checking Date.prototype property of newly constructed objects
includes: [dateConstants.js]
---*/

var x11 = new Date(date_1899_end);

assert.sameValue(
  typeof x11.constructor.prototype,
  "object",
  'The value of `typeof x11.constructor.prototype` is expected to be "object"'
);

var x12 = new Date(date_1899_end);
assert(Date.prototype.isPrototypeOf(x12), 'Date.prototype.isPrototypeOf(x12) must return true');

var x13 = new Date(date_1899_end);

assert.sameValue(
  Date.prototype,
  x13.constructor.prototype,
  'The value of Date.prototype is expected to equal the value of x13.constructor.prototype'
);

var x21 = new Date(date_1900_start);

assert.sameValue(
  typeof x21.constructor.prototype,
  "object",
  'The value of `typeof x21.constructor.prototype` is expected to be "object"'
);

var x22 = new Date(date_1900_start);
assert(Date.prototype.isPrototypeOf(x22), 'Date.prototype.isPrototypeOf(x22) must return true');

var x23 = new Date(date_1900_start);

assert.sameValue(
  Date.prototype,
  x23.constructor.prototype,
  'The value of Date.prototype is expected to equal the value of x23.constructor.prototype'
);

var x31 = new Date(date_1969_end);

assert.sameValue(
  typeof x31.constructor.prototype,
  "object",
  'The value of `typeof x31.constructor.prototype` is expected to be "object"'
);

var x32 = new Date(date_1969_end);
assert(Date.prototype.isPrototypeOf(x32), 'Date.prototype.isPrototypeOf(x32) must return true');

var x33 = new Date(date_1969_end);

assert.sameValue(
  Date.prototype,
  x33.constructor.prototype,
  'The value of Date.prototype is expected to equal the value of x33.constructor.prototype'
);

var x41 = new Date(date_1970_start);

assert.sameValue(
  typeof x41.constructor.prototype,
  "object",
  'The value of `typeof x41.constructor.prototype` is expected to be "object"'
);

var x42 = new Date(date_1970_start);
assert(Date.prototype.isPrototypeOf(x42), 'Date.prototype.isPrototypeOf(x42) must return true');

var x43 = new Date(date_1970_start);

assert.sameValue(
  Date.prototype,
  x43.constructor.prototype,
  'The value of Date.prototype is expected to equal the value of x43.constructor.prototype'
);

var x51 = new Date(date_1999_end);

assert.sameValue(
  typeof x51.constructor.prototype,
  "object",
  'The value of `typeof x51.constructor.prototype` is expected to be "object"'
);

var x52 = new Date(date_1999_end);
assert(Date.prototype.isPrototypeOf(x52), 'Date.prototype.isPrototypeOf(x52) must return true');

var x53 = new Date(date_1999_end);

assert.sameValue(
  Date.prototype,
  x53.constructor.prototype,
  'The value of Date.prototype is expected to equal the value of x53.constructor.prototype'
);

var x61 = new Date(date_2000_start);

assert.sameValue(
  typeof x61.constructor.prototype,
  "object",
  'The value of `typeof x61.constructor.prototype` is expected to be "object"'
);

var x62 = new Date(date_2000_start);
assert(Date.prototype.isPrototypeOf(x62), 'Date.prototype.isPrototypeOf(x62) must return true');

var x63 = new Date(date_2000_start);

assert.sameValue(
  Date.prototype,
  x63.constructor.prototype,
  'The value of Date.prototype is expected to equal the value of x63.constructor.prototype'
);

var x71 = new Date(date_2099_end);

assert.sameValue(
  typeof x71.constructor.prototype,
  "object",
  'The value of `typeof x71.constructor.prototype` is expected to be "object"'
);

var x72 = new Date(date_2099_end);
assert(Date.prototype.isPrototypeOf(x72), 'Date.prototype.isPrototypeOf(x72) must return true');

var x73 = new Date(date_2099_end);

assert.sameValue(
  Date.prototype,
  x73.constructor.prototype,
  'The value of Date.prototype is expected to equal the value of x73.constructor.prototype'
);

var x81 = new Date(date_2100_start);

assert.sameValue(
  typeof x81.constructor.prototype,
  "object",
  'The value of `typeof x81.constructor.prototype` is expected to be "object"'
);

var x82 = new Date(date_2100_start);
assert(Date.prototype.isPrototypeOf(x82), 'Date.prototype.isPrototypeOf(x82) must return true');

var x83 = new Date(date_2100_start);

assert.sameValue(
  Date.prototype,
  x83.constructor.prototype,
  'The value of Date.prototype is expected to equal the value of x83.constructor.prototype'
);
