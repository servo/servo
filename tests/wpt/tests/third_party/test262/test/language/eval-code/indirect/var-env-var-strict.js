// Copyright 2011 Google Inc.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 10.4.2.1_A1
description: >
    Strict indirect eval should not leak top level  declarations into
    the global scope
---*/

if (!('foo' in this)) {
  (1,eval)('"use strict"; var foo = 88;');
  if ('foo' in this) {
    throw new Test262Error("Strict indirect eval leaked a top level declaration");
  }
}
