// Copyright (C) 2026 Caio Lima. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

// Middle's Evaluate is invoked first, while A is still suspended on the
// blocker, and only then is the blocker resolved to let A finish.
import "./middle_FIXTURE.js";
import "./resolve-blocker_FIXTURE.js";

globalThis.evaluations.push("C");
