// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-proxy-object-internal-methods-and-internal-slots-ownpropertykeys
description: >
    If return is not a list object, throw a TypeError exception
info: |
    [[OwnPropertyKeys]] ( )

    7. Let trapResultArray be ? Call(trap, handler, « target »).
    8. Let trapResult be ? CreateListFromArrayLike(trapResultArray, « String,
        Symbol »).
    ...

    #sec-createlistfromarraylike
    7.3.17 CreateListFromArrayLike (obj [, elementTypes] )

    2. If Type(obj) is not Object, throw a TypeError exception.

features: [Proxy, Symbol]
---*/

var target = {};
var p = new Proxy(target, {
  ownKeys: function() {
    return undefined;
  }
});

assert.throws(TypeError, function() {
  Object.keys(p);
});
