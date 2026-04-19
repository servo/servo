// Copyright (C) 2024 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

import "./dep-1.1_FIXTURE.js";
import defer * as ns_1_2 from "./dep-1.2_FIXTURE.js";

globalThis.evaluations.push(1);

export { ns_1_2 };
