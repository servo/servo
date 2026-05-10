// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: |
  RegExp constructor should check the pattern syntax again when adding unicode flag.
info: bugzilla.mozilla.org/show_bug.cgi?id=1274393
esid: pending
---*/

assert.throws(SyntaxError, () => RegExp(/\-/, "u"));
