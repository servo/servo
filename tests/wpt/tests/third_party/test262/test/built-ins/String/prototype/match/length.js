// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.5.4.10_A11
es6id: 21.1.3.11
description: Checking String.prototype.match.length
info: |
    The length property of the match method is 1.

    ES6 Section 17:
    Every built-in Function object, including constructors, has a length
    property whose value is an integer. Unless otherwise specified, this value
    is equal to the largest number of named arguments shown in the subclause
    headings for the function description, including optional parameters.

    [...]

    Unless otherwise specified, the length property of a built-in Function
    object has the attributes { [[Writable]]: false, [[Enumerable]]: false,
    [[Configurable]]: true }.
includes: [propertyHelper.js]
---*/

//////////////////////////////////////////////////////////////////////////////
//CHECK#1
if (!(String.prototype.match.hasOwnProperty("length"))) {
  throw new Test262Error('#1: String.prototype.match.hasOwnProperty("length") return true. Actual: ' + String.prototype.match.hasOwnProperty("length"));
}
//
//////////////////////////////////////////////////////////////////////////////

//////////////////////////////////////////////////////////////////////////////
//CHECK#2
if (String.prototype.match.length !== 1) {
  throw new Test262Error('#2: String.prototype.match.length === 1. Actual: ' + String.prototype.match.length);
}
//
//////////////////////////////////////////////////////////////////////////////

verifyProperty(String.prototype.match, 'length', {
  writable: false,
  enumerable: false,
  configurable: true,
});
