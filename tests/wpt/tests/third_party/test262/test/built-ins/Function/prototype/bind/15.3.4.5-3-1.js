// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.3.4.5-3-1
description: Function.prototype.bind - each arg is defined in A in list order
---*/

var foo = function(x, y) {
  return new Boolean((x + y) === "ab" && arguments[0] === "a" &&
    arguments[1] === "b" && arguments.length === 2);
};

var obj = foo.bind({}, "a", "b");

assert((obj() == true), '(obj() == true) !== true');
