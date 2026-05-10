// Copyright 2018 Mathias Bynens. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
author: Mathias Bynens
description: >
  Non-existent properties must not be supported in Unicode property escapes.
esid: sec-static-semantics-unicodematchproperty-p
negative:
  phase: parse
  type: SyntaxError
features: [regexp-unicode-property-escapes]
---*/

$DONOTEVALUATE();

/\P{Line_Breakz=Alphabetic}/u;
