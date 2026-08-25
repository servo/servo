// Copyright (C) 2024 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: Module graphs with Top-Level Await shouldn't hang.
flags: [module, async]
features: [top-level-await]
---*/

import "./module-graphs-parent-tla_FIXTURE.js";
await import("./module-graphs-grandparent-tla_FIXTURE.js");

$DONE();
