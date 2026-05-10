/*---
description: A module smoketest that fails an assertion.
flags: [module]
---*/
import { a } from "./support/module-helper.js";
assert.sameValue(a, 2, "a should be 2 in module");
