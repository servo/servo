// Copyright (C) 2011 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 13.1
description: >
    let declarations without initialisers in statement positions:
    default : StatementList
---*/
switch (true) { default: let x; }
