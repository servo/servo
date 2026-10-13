// Copyright (C) 2026 Caio Lima. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

import { blocker } from "./setup_FIXTURE.js";

globalThis.evaluations.push("resolve-blocker");
blocker.resolve();
