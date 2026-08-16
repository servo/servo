// Copyright (C) 2026 Caio Lima. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

import defer * as nsD from "./d_FIXTURE.js";

globalThis.evaluations.push("Middle-before-nsD.z");
nsD.z;
globalThis.evaluations.push("Middle-after-nsD.z");
