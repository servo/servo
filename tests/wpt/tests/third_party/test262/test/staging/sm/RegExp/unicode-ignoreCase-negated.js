// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
includes: [compareArray.js]
description: |
  Implement RegExp unicode flag -- ignoreCase flag with negated character class.
info: bugzilla.mozilla.org/show_bug.cgi?id=1135377
esid: pending
---*/

assert.sameValue(/[^A]/iu.exec("A"),
         null);
assert.sameValue(/[^a]/iu.exec("A"),
         null);
assert.sameValue(/[^A]/iu.exec("a"),
         null);
assert.sameValue(/[^a]/iu.exec("a"),
         null);

assert.compareArray(/[^A]/iu.exec("b"),
              ["b"]);
