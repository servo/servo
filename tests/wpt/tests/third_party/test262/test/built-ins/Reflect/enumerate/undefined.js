// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-reflect-object
description: >
  Reflect.enumerate was removed and it's not a function anymore
features: [Reflect]
---*/

assert.sameValue(Reflect.hasOwnProperty("enumerate"), false);
assert.sameValue(Reflect.enumerate, undefined);
