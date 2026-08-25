// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
flags:
  - noStrict
description: |
  pending
esid: pending
---*/
(function() {
    var o = {'arguments': 42};
    with (o) { // Definitely forces heavyweight.
        // Note syntax is not a property access.
        assert.sameValue(delete arguments, true,
                      "arguments property deletion within with block");
    }
    assert.sameValue('arguments' in o, false,
                  "property deletion observable");
})();

(function() {
    var o = {'arguments': 42};
    delete o.arguments;
    assert.sameValue('arguments' in o, false,
                  "arguments property deletion with property access syntax");
})();

(function() {
    var arguments = 42; // Forces heavyweight.
    assert.sameValue(delete arguments, false,
                  "arguments variable");
})();

(function() {
    assert.sameValue(delete arguments, false, "arguments object");
})();
