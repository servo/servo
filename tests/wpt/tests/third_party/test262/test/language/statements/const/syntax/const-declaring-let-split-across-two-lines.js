// Copyright (C) 2015 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
author: Jeff Walden
es6id: 13.3.1.1
description: >
    const: |const let| split across two lines is a static semantics early error.
info: |
  Lexical declarations may not declare a binding named "let".
negative:
  phase: parse
  type: SyntaxError
---*/

$DONOTEVALUATE();

const
let = "irrelevant initializer";
