// Copyright (C) 2015 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: Object.prototype.toLocaleString called with primitive thisValue
info: |
  19.1.3.5 Object.prototype.toLocaleString ( [ reserved1 [ , reserved2 ] ] )

  ...
  2. Return Invoke(O, "toString").
es6id: 19.1.3.5
flags: [onlyStrict]
---*/

Boolean.prototype.toString = function() {
  return typeof this;
};

assert.sameValue(true.toLocaleString(), "boolean");
