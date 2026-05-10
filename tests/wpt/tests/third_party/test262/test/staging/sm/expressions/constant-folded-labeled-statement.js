// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: |
  Constant folder should fold labeled statements
info: bugzilla.mozilla.org/show_bug.cgi?id=1499448
esid: pending
---*/

if (typeof disassemble === "function") {
    var code = disassemble(() => { x: 2+2; });

    assert.sameValue(true, /Int8 4/.test(code));
}

