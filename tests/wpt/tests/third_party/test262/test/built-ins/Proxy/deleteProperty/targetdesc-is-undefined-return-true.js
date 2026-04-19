// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 9.5.10
description: >
    [[Delete]] (P)

    14. If targetDesc is undefined, return true.
features: [Proxy]
---*/

var p = new Proxy({}, {
  deleteProperty: function() {
    return true;
  }
});

assert.sameValue(delete p.attr, true);
