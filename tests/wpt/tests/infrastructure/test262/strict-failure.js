/*---
description: A strict mode smoketest that fails an assertion.
flags: [onlyStrict]
---*/
"use strict";
assert.sameValue(1, 2, "One should be two in strict mode");
