// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    When the hasOwnProperty method is called with argument V, the following steps are taken:
    i) Let O be this object
    ii) Call ToString(V)
    iii) If O doesn't have a property with the name given by Result(ii), return false
    iv) Return true
es5id: 15.2.4.5_A1_T3
description: >
    Argument of the hasOwnProperty method is a custom property of a
    function object
---*/

var FACTORY = function() {
  this.aproperty = 1;
};

var instance = new FACTORY;

assert.sameValue(
  typeof Object.prototype.hasOwnProperty,
  "function",
  'The value of `typeof Object.prototype.hasOwnProperty` is expected to be "function"'
);

assert.sameValue(
  typeof instance.hasOwnProperty,
  "function",
  'The value of `typeof instance.hasOwnProperty` is expected to be "function"'
);

assert(
  !instance.hasOwnProperty("toString"),
  'The value of !instance.hasOwnProperty("toString") is expected to be true'
);

assert(
  !!instance.hasOwnProperty("aproperty"),
  'The value of !!instance.hasOwnProperty("aproperty") is expected to be true'
);
