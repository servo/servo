// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.4-4-40
description: >
    Object.getOwnPropertyNames - inherited data property of String
    object 'O' is not pushed into the returned array
---*/

var str = new String("abc");

String.prototype.protoProperty = "protoString";

var result = Object.getOwnPropertyNames(str);

for (var p in result) {
  assert.notSameValue(result[p], "protoProperty", 'result[p]');
}
