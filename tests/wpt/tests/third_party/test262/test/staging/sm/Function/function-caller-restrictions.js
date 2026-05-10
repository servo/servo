// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
flags:
  - noStrict
description: |
  pending
esid: pending
---*/

// Function#caller restrictions as proposed by
// https://github.com/claudepache/es-legacy-function-reflection/

function caller() {
    return caller.caller;
}

assert.sameValue(caller(), null);
assert.sameValue(Reflect.apply(caller, undefined, []), null);

assert.sameValue([0].map(caller)[0], null);

(function strict() {
    "use strict";
    assert.sameValue(caller(), null);
})();

(async function() {
    assert.sameValue(caller(), null);
})();

assert.sameValue(function*() {
    yield caller();
}().next().value, null);
