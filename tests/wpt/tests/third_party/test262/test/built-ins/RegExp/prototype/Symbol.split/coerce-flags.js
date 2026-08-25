// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es6id: 21.2.5.11
description: String coercion of `flags` property
info: |
    [...]
    7. Let flags be ToString(Get(rx, "flags")).
    [...]
    13. Let splitter be Construct(C, «rx, newFlags»).
    [...]
features: [Symbol.split, Symbol.species]
---*/

var obj = {
  constructor: function() {},
  flags: {
    toString: function() {
      return 'toString valuey';
    }
  }
};
var flagsArg;

obj.constructor = function() {};
obj.constructor[Symbol.species] = function(_, flags) {
  flagsArg = flags;
  return /./y;
};

RegExp.prototype[Symbol.split].call(obj);

assert.sameValue(flagsArg, 'toString valuey');
