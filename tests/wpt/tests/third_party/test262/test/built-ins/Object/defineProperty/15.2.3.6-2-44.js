// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.6-2-44
description: >
    Object.defineProperty - argument 'P' is an object that has an own
    valueOf method
---*/

var obj = {};

var ownProp = {
  valueOf: function() {
    return "abc";
  },
  toString: undefined
};

Object.defineProperty(obj, ownProp, {});

assert(obj.hasOwnProperty("abc"), 'obj.hasOwnProperty("abc") !== true');
