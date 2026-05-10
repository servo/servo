// Copyright (C) 2024 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: module graphs with TLA shouldn't hang
flags: [module, async]
features: [top-level-await]
---*/

import "./parent-tla_FIXTURE.js";
await import("./grandparent-tla_FIXTURE.js");

$DONE();
