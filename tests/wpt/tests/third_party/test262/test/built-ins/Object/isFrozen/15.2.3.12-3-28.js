// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.12-3-28
description: >
    Object.isFrozen returns true when all own properties of 'O' are
    not writable and not configurable, and 'O' is not extensible
---*/

var obj = {};

Object.defineProperty(obj, "foo1", {
  value: 20,
  writable: false,
  enumerable: false,
  configurable: false
});


function get_func() {
  return 10;
}

function set_func() {}

Object.defineProperty(obj, "foo2", {
  get: get_func,
  set: set_func,
  configurable: false
});

Object.preventExtensions(obj);

assert(Object.isFrozen(obj), 'Object.isFrozen(obj) !== true');
