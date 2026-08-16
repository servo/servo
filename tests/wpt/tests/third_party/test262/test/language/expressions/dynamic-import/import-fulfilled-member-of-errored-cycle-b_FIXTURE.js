// Copyright (C) 2026 Caio Lima. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

import "./import-fulfilled-member-of-errored-cycle-c_FIXTURE.js";

await Promise.resolve(0);

throw new Error("async error in B");
