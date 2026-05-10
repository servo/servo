// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

import "./pending-async-dep-from-cycle_cycle-leaf_FIXTURE.js";

globalThis.logs.push("cycle root start");
await 1;
globalThis.logs.push("cycle root end");
