/*---
description: A module smoketest that throws an unexpected ReferenceError.
flags: [module]
---*/
import { a } from "./support/module-helper.js";
foo.bar();
