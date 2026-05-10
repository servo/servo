// Copyright (C) 2024 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

import defer * as dep2 from "./dep-2_FIXTURE.js";

globalThis.dep3evaluated = false;

try {
  dep2.foo;
} catch (error) {
  globalThis["evaluating dep2.foo error"] = error;
}

globalThis["evaluating dep2.foo evaluates dep3"] = globalThis.dep3evaluated;
