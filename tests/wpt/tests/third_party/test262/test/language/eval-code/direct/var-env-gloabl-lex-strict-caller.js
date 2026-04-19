// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-evaldeclarationinstantiation
description: No variable collision with global lexical binding
info: |
    [...]
    5. If strict is false, then
    [...]
flags: [onlyStrict]
features: [let]
---*/

let x;

eval('var x;');
