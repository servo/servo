// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    When the hasOwnProperty method is called with argument V, the following steps are taken:
    i) Let O be this object
    ii) Call ToString(V)
    iii) If O doesn't have a property with the name given by Result(ii), return false
    iv) Return true
es5id: 15.2.4.5_A1_T1
description: >
    Checking type of the Object.prototype.hasOwnProperty and the
    returned result
---*/
assert.sameValue(
  typeof Object.prototype.hasOwnProperty,
  "function",
  'The value of `typeof Object.prototype.hasOwnProperty` is expected to be "function"'
);

assert(
  !!Object.prototype.hasOwnProperty("hasOwnProperty"),
  'The value of !!Object.prototype.hasOwnProperty("hasOwnProperty") is expected to be true'
);
