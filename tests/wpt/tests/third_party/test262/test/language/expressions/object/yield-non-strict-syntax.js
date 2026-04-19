// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 12.2.5
description: >
    In non-strict mode, yield is a valid Identifier.
flags: [noStrict]
---*/
var yield = 1;
var object = {yield};
