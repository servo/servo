// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: >
    In the absence of the "use strict" directive, generator functions declared
    as methods obey "global" ThisMode semantics.
es6id: 14.4.13
flags: [noStrict]
features: [generators]
---*/

var global = (function() { return this; }());
var thisValue = null;
var method = {
  *method() {
    thisValue = this;
  }
}.method;

method().next();

assert.sameValue(thisValue, global);
