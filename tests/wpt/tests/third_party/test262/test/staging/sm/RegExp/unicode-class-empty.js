// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
includes: [compareArray.js]
description: |
  Implement RegExp unicode flag -- empty class should not match anything.
info: bugzilla.mozilla.org/show_bug.cgi?id=1135377
esid: pending
---*/

assert.sameValue(/[]/u.exec("A"),
         null);
assert.sameValue(/[]/u.exec("\uD83D"),
         null);
assert.sameValue(/[]/u.exec("\uDC38"),
         null);
assert.sameValue(/[]/u.exec("\uD83D\uDC38"),
         null);

assert.compareArray(/[^]/u.exec("A"),
              ["A"]);
assert.compareArray(/[^]/u.exec("\uD83D"),
              ["\uD83D"]);
assert.compareArray(/[^]/u.exec("\uDC38"),
              ["\uDC38"]);
assert.compareArray(/[^]/u.exec("\uD83D\uDC38"),
              ["\uD83D\uDC38"]);
