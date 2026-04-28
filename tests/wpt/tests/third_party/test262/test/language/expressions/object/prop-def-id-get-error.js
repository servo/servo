// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 12.2.5.9
description: >
    Errors thrown during IdentifierReference value retrieval are forwarded to
    the runtime.
---*/

assert.throws(ReferenceError, function() {
  ({ unresolvable });
});
