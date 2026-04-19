// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    The propertyIsEnumerable method does not consider objects in the
    prototype chain
es5id: 15.2.4.7_A1_T1
description: >
    Calling the propertyIsEnumerable method for object in the
    prototype chain
---*/
assert.sameValue(
  typeof Object.prototype.propertyIsEnumerable,
  "function",
  'The value of `typeof Object.prototype.propertyIsEnumerable` is expected to be "function"'
);

var proto = {
  rootprop: "avis"
};

function AVISFACTORY(name) {
  this.name = name
}

AVISFACTORY.prototype = proto;

var seagull = new AVISFACTORY("seagull");

assert.sameValue(
  typeof seagull.propertyIsEnumerable,
  "function",
  'The value of `typeof seagull.propertyIsEnumerable` is expected to be "function"'
);

assert(
  !!seagull.propertyIsEnumerable("name"),
  'The value of !!seagull.propertyIsEnumerable("name") is expected to be true'
);

assert(
  !seagull.propertyIsEnumerable("rootprop"),
  'The value of !seagull.propertyIsEnumerable("rootprop") is expected to be true'
);
