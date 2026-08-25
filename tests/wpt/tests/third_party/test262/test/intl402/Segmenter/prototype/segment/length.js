// Copyright 2018 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-Intl.Segmenter.prototype.segment
description: Checks the "length" property of Intl.Segmenter.prototype.segment().
info: |
    Unless specified otherwise in this document, the objects, functions, and constructors described in this standard are subject to the generic requirements and restrictions specified for standard built-in ECMAScript objects in the ECMAScript 2019 Language Specification, 10th edition, clause 17, or successor.
    The Segmenter constructor is a standard built-in property of the Intl object.
    Every built-in function object, including constructors, has a length property whose value is an integer. Unless otherwise specified, this value is equal to the largest number of named arguments shown in the subclause headings for the function description. Optional parameters (which are indicated with brackets: [ ]) or rest parameters (which are shown using the form «...name») are not included in the default argument count.
    Unless otherwise specified, the length property of a built-in function object has the attributes { [[Writable]]: false, [[Enumerable]]: false, [[Configurable]]: true }.
includes: [propertyHelper.js]
features: [Intl.Segmenter]
---*/

verifyProperty(Intl.Segmenter.prototype.segment, "length", {
  value: 1,
  writable: false,
  enumerable: false,
  configurable: true
});
