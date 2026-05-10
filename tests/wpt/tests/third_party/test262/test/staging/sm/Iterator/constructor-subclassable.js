// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
  Iterator constructor can be subclassed.

  Iterator is not enabled unconditionally
features:
  - iterator-helpers
description: |
  pending
esid: pending
---*/
class TestIterator extends Iterator {
}

assert.sameValue(new TestIterator() instanceof Iterator, true);

