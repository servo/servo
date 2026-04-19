// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.4-4-36
description: >
    Object.getOwnPropertyNames - inherited data properties are not
    pushed into the returned array
---*/

var proto = {
  "parent": "parent"
};

var Con = function() {};
Con.prototype = proto;

var child = new Con();

var result = Object.getOwnPropertyNames(child);

for (var p in result) {
  assert.notSameValue(result[p], "parent", 'result[p]');
}
