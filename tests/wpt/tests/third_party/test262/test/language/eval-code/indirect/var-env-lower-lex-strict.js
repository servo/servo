// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-evaldeclarationinstantiation
description: No variable collision with lexical binding in lower scope
info: |
    [...]
    5. If strict is false, then
    [...]
features: [let]
---*/

{
  let x;
  {
    (0,eval)('"use strict"; var x;');
  }
}
