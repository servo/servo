// Copyright 2020 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-%segmentsprototype%.containing
description: Checks the "name" property of %Segments.prototype%.containing ( index )
info: |
    %Segments.prototype%.containing ( index )
    Unless specified otherwise in this document, the objects, functions, and constructors described in this standard are subject to the generic requirements and restrictions specified for standard built-in ECMAScript objects in the ECMAScript 2020 Language Specification, 11th edition, clause 17, or successor.
    Every built-in function object, including constructors, that is not identified as an anonymous function has a name property whose value is a String. Unless otherwise specified, this value is the name that is given to the function in this specification.
    Unless otherwise specified, the name property of a built-in function object, if it exists, has the attributes { [[Writable]]: false, [[Enumerable]]: false, [[Configurable]]: true }.

includes: [propertyHelper.js]
features: [Intl.Segmenter]
---*/
const segment = (new Intl.Segmenter()).segment("");
verifyProperty(segment.containing, "name", {
  value: "containing",
  writable: false,
  enumerable: false,
  configurable: true
});
