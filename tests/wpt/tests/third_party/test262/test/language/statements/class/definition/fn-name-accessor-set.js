// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es6id: 14.3.9
description: Assignment of function `name` attribute ("set" accessor)
info: |
    MethodDefinition :
        set PropertyName ( PropertySetParameterList ) { FunctionBody }

    [...]
    7. Perform SetFunctionName(closure, propKey, "set").
includes: [propertyHelper.js]
features: [Symbol]
---*/

var namedSym = Symbol('test262');
var anonSym = Symbol();
var setter;

class A {
  set id(_) {}
  set [anonSym](_) {}
  set [namedSym](_) {}
  static set id(_) {}
  static set [anonSym](_) {}
  static set [namedSym](_) {}
}

setter = Object.getOwnPropertyDescriptor(A.prototype, 'id').set;
verifyProperty(setter, 'name', {
  value: 'set id',
  writable: false,
  enumerable: false,
  configurable: true,
});

setter = Object.getOwnPropertyDescriptor(A.prototype, anonSym).set;
verifyProperty(setter, 'name', {
  value: 'set ',
  writable: false,
  enumerable: false,
  configurable: true,
});

setter = Object.getOwnPropertyDescriptor(A.prototype, namedSym).set;
verifyProperty(setter, 'name', {
  value: 'set [test262]',
  writable: false,
  enumerable: false,
  configurable: true,
});

setter = Object.getOwnPropertyDescriptor(A, 'id').set;
verifyProperty(setter, 'name', {
  value: 'set id',
  writable: false,
  enumerable: false,
  configurable: true,
});

setter = Object.getOwnPropertyDescriptor(A, anonSym).set;
verifyProperty(setter, 'name', {
  value: 'set ',
  writable: false,
  enumerable: false,
  configurable: true,
});

setter = Object.getOwnPropertyDescriptor(A, namedSym).set;
verifyProperty(setter, 'name', {
  value: 'set [test262]',
  writable: false,
  enumerable: false,
  configurable: true,
});
