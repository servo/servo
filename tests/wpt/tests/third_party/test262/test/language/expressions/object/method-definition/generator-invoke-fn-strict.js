// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: >
    In the presence of the "use strict" directive, generator functions declared
    as methods obey "strict" ThisMode semantics.
es6id: 14.4.13
flags: [noStrict]
features: [generators]
---*/

var thisValue = null;
var method = {
  *method() {
    'use strict';
    thisValue = this;
  }
}.method;

method().next();

assert.sameValue(thisValue, undefined);
