// Copyright (C) 2021 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-intl.supportedvaluesof
description: >
  Intl.supportedValuesOf.name value and descriptor.
info: |
  Intl.supportedValuesOf ( key )

  18 ECMAScript Standard Built-in Objects:
    Every built-in function object, including constructors, has a "name"
    property whose value is a String. Unless otherwise specified, this value is
    the name that is given to the function in this specification. Functions that
    are identified as anonymous functions use the empty String as the value of
    the "name" property. For functions that are specified as properties of
    objects, the name value is the property name string used to access the
    function.

    Unless otherwise specified, the "name" property of a built-in function object
    has the attributes { [[Writable]]: false, [[Enumerable]]: false,
    [[Configurable]]: true }.
includes: [propertyHelper.js]
features: [Intl-enumeration]
---*/

verifyProperty(Intl.supportedValuesOf, "name", {
  value: "supportedValuesOf",
  writable: false,
  enumerable: false,
  configurable: true,
});
