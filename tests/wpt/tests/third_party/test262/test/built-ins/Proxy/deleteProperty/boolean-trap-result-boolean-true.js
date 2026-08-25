// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 9.5.10
description: >
    [[Delete]] (P)

    The result is a Boolean value.
features: [Proxy, Reflect]
---*/

var p = new Proxy({}, {
  deleteProperty: function() {
    return 1;
  }
});

assert.sameValue(Reflect.deleteProperty(p, "attr"), true);
