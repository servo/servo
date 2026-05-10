// Copyright (C) 2013 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: >
    `yield` is not a reserved identifier in non-strict mode code and may be
    used as a label.
es6id: 12.1.1
flags: [noStrict]
---*/

yield: 1;
