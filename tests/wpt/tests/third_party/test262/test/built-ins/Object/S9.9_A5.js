// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    ToObject conversion from String: create a new String object
    whose [[value]] property is set to the value of the string
es5id: 9.9_A5
description: Converting from various strings to Object
---*/
assert.sameValue(
  Object("some string").valueOf(),
  "some string",
  'Object("some string").valueOf() must return "some string"'
);

assert.sameValue(
  typeof Object("some string"),
  "object",
  'The value of `typeof Object("some string")` is expected to be "object"'
);

assert.sameValue(
  Object("some string").constructor.prototype,
  String.prototype,
  'The value of Object("some string").constructor.prototype is expected to equal the value of String.prototype'
);

assert.sameValue(Object("").valueOf(), "", 'Object("").valueOf() must return ""');
assert.sameValue(typeof Object(""), "object", 'The value of `typeof Object("")` is expected to be "object"');

assert.sameValue(
  Object("").constructor.prototype,
  String.prototype,
  'The value of Object("").constructor.prototype is expected to equal the value of String.prototype'
);

assert.sameValue(Object("\r\t\b\n\v\f").valueOf(), "\r\t\b\n\v\f", 'Object("rtbnvf").valueOf() must return "rtbnvf"');

assert.sameValue(
  typeof Object("\r\t\b\n\v\f"),
  "object",
  'The value of `typeof Object("rtbnvf")` is expected to be "object"'
);

assert.sameValue(
  Object("\r\t\b\n\v\f").constructor.prototype,
  String.prototype,
  'The value of Object("rtbnvf").constructor.prototype is expected to equal the value of String.prototype'
);

assert.sameValue(Object(String(10)).valueOf(), "10", 'Object(String(10)).valueOf() must return "10"');

assert.sameValue(
  typeof Object(String(10)),
  "object",
  'The value of `typeof Object(String(10))` is expected to be "object"'
);

assert.sameValue(
  Object(String(10)).constructor.prototype,
  String.prototype,
  'The value of Object(String(10)).constructor.prototype is expected to equal the value of String.prototype'
);
