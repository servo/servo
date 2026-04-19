// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    When the propertyIsEnumerable method is called with argument V, the following steps are taken:
    i) Let O be this object
    ii) Call ToString(V)
    iii) If O doesn't have a property with the name given by Result(ii), return false
    iv) If the property has the DontEnum attribute, return false
    v) Return true
es5id: 15.2.4.7_A2_T2
description: >
    Argument of the propertyIsEnumerable method is a custom boolean
    property
---*/
assert.sameValue(
  typeof Object.prototype.propertyIsEnumerable,
  "function",
  'The value of `typeof Object.prototype.propertyIsEnumerable` is expected to be "function"'
);

var obj = {
  the_property: true
};

assert.sameValue(
  typeof obj.propertyIsEnumerable,
  "function",
  'The value of `typeof obj.propertyIsEnumerable` is expected to be "function"'
);

assert(
  !!obj.propertyIsEnumerable("the_property"),
  'The value of !!obj.propertyIsEnumerable("the_property") is expected to be true'
);

var accum = "";
for (var prop in obj) {
  accum += prop;
}
assert.sameValue(accum.indexOf("the_property"), 0, 'accum.indexOf("the_property") must return 0');

// TODO: Convert to verifyProperty() format.
