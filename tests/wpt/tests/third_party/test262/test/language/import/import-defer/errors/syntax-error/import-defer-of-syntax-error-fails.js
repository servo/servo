// Copyright (C) 2024 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-module-namespace-exotic-objects-get-p-receiver-EnsureDeferredNamespaceEvaluation
description: >
  Syntax errors in deferred modules are reported eagerly
info: |
  LoadRequestedModules ([ _hostDefined_ ])
    - just notice that it does not check if the module is deferred

flags: [module]
features: [import-defer]

negative:
  phase: resolution
  type: SyntaxError
---*/

$DONOTEVALUATE();

import defer * as ns from "./syntax-error_FIXTURE.js";
