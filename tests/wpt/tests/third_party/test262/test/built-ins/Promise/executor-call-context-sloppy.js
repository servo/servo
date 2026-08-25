// Copyright 2014 Cubane Canada, Inc. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-promise-executor
author: Sam Mikes
description: >
  Promise executor is called in global object context in sloppy mode.
info: |
  25.6.3.1 Promise ( executor )

  [...]
  9. Let completion be Call(executor, undefined, « resolvingFunctions.[[Resolve]], resolvingFunctions.[[Reject]] »).
flags: [noStrict]
---*/

var _this;

new Promise(function() {
  _this = this;
});

assert.sameValue(_this, this);
