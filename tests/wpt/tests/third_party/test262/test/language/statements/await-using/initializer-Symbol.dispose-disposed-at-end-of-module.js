// Copyright (C) 2026 Kevin Gibbons. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-source-text-module-record-execute-module
description: Initialized value is disposed at end of Module (Symbol.dispose)
flags: [module, async]
features: [explicit-resource-management, top-level-await]
---*/

var disposed = false;
var resource = {
  [Symbol.dispose]() {
    assert.sameValue(disposed, false, 'disposal should happen once');
    disposed = true;
    $DONE();
  }
};

await using _ = resource;

assert.sameValue(disposed, false, 'resources should not be disposed until module evaluation finishes');
