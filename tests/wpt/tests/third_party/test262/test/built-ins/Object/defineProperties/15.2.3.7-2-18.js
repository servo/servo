// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.7-2-18
description: >
    Object.defineProperties - argument 'Properties' is the global
    object
---*/

var global = this;
var obj = {};
var result = false;

try {
  Object.defineProperty(this, "prop", {
    get: function() {
      result = (this === global);
      return {};
    },
    enumerable: true,
    configurable: true
  });

  Object.defineProperties(obj, this);
} catch (e) {
  if (!(e instanceof TypeError)) {
    throw new Test262Error("Expected TypeError, got " + e);
  }
  result = true;
} finally {
  delete this.prop;
}

assert(result, 'result !== true');
