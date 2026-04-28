// Copyright (C) 2023 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-innermoduleevaluation
description: >
  While an asynchronous module is waiting for a promise resolution,
  sibling modules in the modules graph must be evaluated.
flags: [module, async]
features: [top-level-await]
---*/

import "./async-module-tla_FIXTURE.js";
import { check } from "./async-module-sync_FIXTURE.js";
assert.sameValue(check, false);
$DONE();
