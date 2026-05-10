// Copyright (C) 2014 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-gettemplateobject
description: Properties of the template object
info: |
    The first argument to a tagged template should be a template object as
    defined by the GetTemplateObject abstract operation.
includes: [propertyHelper.js]
---*/
var templateObject

function tag(parameter) {
  templateObject = parameter;
}

tag`${1}`;

assert(Array.isArray(templateObject.raw), 'The template object is an array');

verifyProperty(templateObject, 'raw', {
  enumerable: false,
  writable: false,
  configurable: false,
});

assert(Array.isArray(templateObject), 'The "raw" object is an array');

verifyProperty(templateObject, '0', {
  enumerable: true,
  writable: false,
  configurable: false,
});

verifyProperty(templateObject, 'length', {
  enumerable: false,
  writable: false,
  configurable: false,
});

verifyProperty(templateObject.raw, '0', {
  enumerable: true,
  writable: false,
  configurable: false,
});

verifyProperty(templateObject.raw, 'length', {
  enumerable: false,
  writable: false,
  configurable: false,
});
