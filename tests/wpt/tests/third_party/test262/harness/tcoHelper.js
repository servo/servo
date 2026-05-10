// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: |
    This defines the number of consecutive recursive function calls that must be
    made in order to prove that stack frames are properly destroyed according to
    ES2015 tail call optimization semantics.
defines: [$MAX_ITERATIONS]
---*/




var $MAX_ITERATIONS = 100000;
