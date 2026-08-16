// Copyright (C) 2026 Caio Lima. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

import { blocker, aStarted } from "./setup_FIXTURE.js";
import "./b_FIXTURE.js";

globalThis.evaluations.push("A-before-await");
aStarted.resolve();
await blocker.promise;
globalThis.evaluations.push("A-after-await");
