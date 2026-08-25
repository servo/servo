// Copyright (C) 2024 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

import { first, third, resolveSecond, rejectDone, resolveDone } from "./promises_FIXTURE.js";
import defer * as ns from "./dep-1-tla_FIXTURE.js";

// ns is now in the ~evaluating~ state
try {
  ns.foo;
} catch (error) {
  globalThis["error on ns.foo while evaluating"] = error;
}

first.then(() => {
  // ns is now in the ~evaluating-async~ state
  try {
    ns.foo;
  } catch (error) {
    globalThis["error on ns.foo while evaluating-async"] = error;
  }
  resolveSecond();
}).then(() => {
  return third.then(() => {
    // ns is now in the ~evaluated~ state
    let foo = ns.foo;
    globalThis["value of ns.foo when evaluated"] = foo;
  })
}).then(resolveDone, rejectDone);
