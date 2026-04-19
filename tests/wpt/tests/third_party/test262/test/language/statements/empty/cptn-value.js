// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-empty-statement-runtime-semantics-evaluation
es6id: 13.4.1
description: Returns an empty completion
info: |
  1. Return NormalCompletion(empty).
---*/

assert.sameValue(eval(';'), undefined);
assert.sameValue(eval('2;;'), 2);
assert.sameValue(eval('3;;;'), 3);
