// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: |
  pending
esid: pending
---*/
(function() {
    var rx = /a/g;
    var b = {
        get a() {
            rx.compile("b");
            return "A";
        }
    };

    // Replacer function which is applicable for the elem-base optimization in
    // RegExp.prototype.@@replace.
    function replacer(a) {
        return b[a];
    }

    var result = rx[Symbol.replace]("aaa", replacer);

    assert.sameValue(result, "AAA");
})();

