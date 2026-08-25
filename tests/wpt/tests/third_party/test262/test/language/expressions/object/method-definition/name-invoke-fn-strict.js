// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: >
    In the presence of the "use strict" directive, functions declared as
    methods obey "strict" ThisMode semantics.
es6id: 14.3.8
flags: [noStrict]
---*/

var thisValue = null;
var method = {
  method() {
    'use strict';
    thisValue = this;
  }
}.method;

method();

assert.sameValue(thisValue, undefined);
