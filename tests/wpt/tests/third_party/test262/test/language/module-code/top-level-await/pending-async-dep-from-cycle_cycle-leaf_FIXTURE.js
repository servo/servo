// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

import "./pending-async-dep-from-cycle_cycle-root_FIXTURE.js";

globalThis.logs.push("cycle leaf start");
await 1;
globalThis.logs.push("cycle leaf end");
