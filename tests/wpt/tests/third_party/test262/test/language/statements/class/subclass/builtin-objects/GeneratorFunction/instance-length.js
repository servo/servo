// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 25.2.4.1
description: >
  Subclassed GeneratorFunction instances `length` property
info: |
  25.2.4.1 length

  The value of the length property is an integer that indicates the typical
  number of arguments expected by the GeneratorFunction. However, the language
  permits the function to be invoked with some other number of arguments. The
  behaviour of a GeneratorFunction when invoked on a number of arguments other
  than the number specified by its length property depends on the function.

  This property has the attributes { [[Writable]]: false, [[Enumerable]]: false,
  [[Configurable]]: true }.
includes: [propertyHelper.js]
---*/

var GeneratorFunction = Object.getPrototypeOf(function* () {}).constructor;

class GFn extends GeneratorFunction {}

var gfn = new GFn('a', 'b', 'return a + b');

verifyProperty(gfn, 'length', {
  value: 2,
  writable: false,
  enumerable: false,
  configurable: true,
});
