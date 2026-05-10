// Copyright (C) 2019 Alexey Shvayka. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-promise.prototype.finally
description: >
  Promise.prototype.finally invoked on thenable returns result of "then" call.
features: [Promise.prototype.finally]
---*/

var thenResult = {};
var Thenable = function() {};
Thenable.prototype.then = function() { return thenResult; };

assert.sameValue(
  Promise.prototype.finally.call(new Thenable()),
  thenResult
);
