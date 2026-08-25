// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es6id: 14.4.13
description: >
    Assignment of function `name` attribute (GeneratorMethod)
info: |
    GeneratorMethod :
        * PropertyName ( StrictFormalParameters ) { GeneratorBody }

    [...]
    9. Perform SetFunctionName(closure, propKey).
includes: [propertyHelper.js]
features: [generators, Symbol]
---*/

var namedSym = Symbol('test262');
var anonSym = Symbol();

class A {
  *id() {}
  *[anonSym]() {}
  *[namedSym]() {}
  static *id() {}
  static *[anonSym]() {}
  static *[namedSym]() {}
}

verifyProperty(A.prototype.id, 'name', {
  value: 'id',
  writable: false,
  enumerable: false,
  configurable: true,
});

verifyProperty(A.prototype[anonSym], 'name', {
  value: '',
  writable: false,
  enumerable: false,
  configurable: true,
});

verifyProperty(A.prototype[namedSym], 'name', {
  value: '[test262]',
  writable: false,
  enumerable: false,
  configurable: true,
});

verifyProperty(A.id, 'name', {
  value: 'id',
  writable: false,
  enumerable: false,
  configurable: true,
});

verifyProperty(A[anonSym], 'name', {
  value: '',
  writable: false,
  enumerable: false,
  configurable: true,
});

verifyProperty(A[namedSym], 'name', {
  value: '[test262]',
  writable: false,
  enumerable: false,
  configurable: true,
});
