// Copyright (C) 2026 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

import "./setup_FIXTURE.js";
import defer * as ns from "./dep2_FIXTURE.js";

globalThis.evaluations.push("reexport");

export { ns };
