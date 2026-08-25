// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    When the hasOwnProperty method is called with argument V, the following steps are taken:
    i) Let O be this object
    ii) Call ToString(V)
    iii) If O doesn't have a property with the name given by Result(ii), return false
    iv) Return true
es5id: 15.2.4.5_A1_T2
description: Argument of the hasOwnProperty method is a custom boolean property
---*/
assert.sameValue(
  typeof Object.prototype.hasOwnProperty,
  "function",
  'The value of `typeof Object.prototype.hasOwnProperty` is expected to be "function"'
);

var obj = {
  the_property: true
};

assert.sameValue(
  typeof obj.hasOwnProperty,
  "function",
  'The value of `typeof obj.hasOwnProperty` is expected to be "function"'
);

assert(
  !obj.hasOwnProperty("hasOwnProperty"),
  'The value of !obj.hasOwnProperty("hasOwnProperty") is expected to be true'
);

assert(
  !!obj.hasOwnProperty("the_property"),
  'The value of !!obj.hasOwnProperty("the_property") is expected to be true'
);
