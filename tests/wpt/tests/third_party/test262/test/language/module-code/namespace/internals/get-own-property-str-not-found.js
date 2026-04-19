// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-module-namespace-exotic-objects-getownproperty-p
description: >
    Behavior of the [[GetOwnProperty]] internal method with a string argument
    describing a binding that cannot be found
info: |
    1. If Type(P) is Symbol, return OrdinaryGetOwnProperty(O, P).
    2. Let exports be the value of O's [[Exports]] internal slot.
    3. If P is not an element of exports, return undefined.
flags: [module]
---*/

import * as ns from './get-own-property-str-not-found.js';
var test262;
export { test262 as anotherName };
var desc;

assert.sameValue(
  Object.prototype.hasOwnProperty.call(ns, 'test262'),
  false,
  'hasOwnProperty: test262'
);
desc = Object.getOwnPropertyDescriptor(ns, 'test262');
assert.sameValue(desc, undefined, 'property descriptor for "test262"');

assert.sameValue(
  Object.prototype.hasOwnProperty.call(ns, 'desc'),
  false,
  'hasOwnProperty: desc'
);
desc = Object.getOwnPropertyDescriptor(ns, 'desc');
assert.sameValue(desc, undefined, 'property descriptor for "desc"');

assert.sameValue(
  Object.prototype.hasOwnProperty.call(ns, 'toStringTag'),
  false,
  'hasOwnProperty: toStringTag'
);
desc = Object.getOwnPropertyDescriptor(ns, 'toStringTag');
assert.sameValue(desc, undefined, 'property descriptor for "toStringTag"');

assert.sameValue(
  Object.prototype.hasOwnProperty.call(ns, 'iterator'),
  false,
  'hasOwnProperty: iterator'
);
desc = Object.getOwnPropertyDescriptor(ns, 'iterator');
assert.sameValue(desc, undefined, 'property descriptor for "iterator"');

assert.sameValue(
  Object.prototype.hasOwnProperty.call(ns, '__proto__'),
  false,
  'hasOwnProperty: __proto__'
);
desc = Object.getOwnPropertyDescriptor(ns, '__proto__');
assert.sameValue(desc, undefined, 'property descriptor for "__proto__"');

assert.sameValue(
  Object.prototype.hasOwnProperty.call(ns, 'default'),
  false,
  'hasOwnProperty: default'
);
desc = Object.getOwnPropertyDescriptor(ns, 'default');
assert.sameValue(desc, undefined, 'property descriptor for "default"');
