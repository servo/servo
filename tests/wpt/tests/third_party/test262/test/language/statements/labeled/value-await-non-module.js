// Copyright (C) 2017 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-identifiers-static-semantics-early-errors
description: >
  `await` is not a reserved identifier in non-module code and may be used as a label.
info: |
  LabelIdentifier : await

  It is a Syntax Error if the goal symbol of the syntactic grammar is Module.
---*/

await: 1;
