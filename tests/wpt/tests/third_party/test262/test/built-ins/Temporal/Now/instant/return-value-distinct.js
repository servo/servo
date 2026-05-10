// Copyright (C) 2021 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-temporal.now.instant
description: Each invocation of the function produces a distinct object value
features: [Temporal]
---*/

var instant1 = Temporal.Now.instant();
var instant2 = Temporal.Now.instant();

assert.notSameValue(instant1, instant2, 'The value of instant1 is expected to not equal the value of `instant2`');
