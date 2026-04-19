// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: >
    Positive and negative zero satisfy the assertion.
---*/

assert.notSameValue(0, -0);
