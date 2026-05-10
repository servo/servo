// Copyright (C) 2015 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: >
    Generator expressions cannot be used as constructors.
es6id: 14.4
features: [generators]
---*/

var g = function*(){};

assert.throws(TypeError, function() {
  var instance = new g();
});
