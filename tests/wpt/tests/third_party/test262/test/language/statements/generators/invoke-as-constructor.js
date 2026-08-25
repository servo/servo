// Copyright (C) 2015 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: >
    Generator statements cannot be used as constructors.
es6id: 14.4
features: [generators]
---*/

function* g(){}

assert.throws(TypeError, function() {
  var instance = new g();
});
