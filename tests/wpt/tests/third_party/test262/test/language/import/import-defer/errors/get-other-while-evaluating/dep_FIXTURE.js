// Copyright (C) 2024 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

import defer * as main from "./main.js";

try {
  main.foo;
} catch (error) {
  globalThis["error on main.foo"] = error;
}
