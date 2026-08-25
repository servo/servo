// Copyright 2018 Mathias Bynens. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
author: Mathias Bynens
description: >
  Unicode property escapes must be supported in character classes.
esid: sec-static-semantics-unicodematchproperty-p
features: [regexp-unicode-property-escapes]
---*/

/[\p{Hex}]/u;

assert(
  /[\p{Hex}\P{Hex}]/u.test('\u{1D306}'),
  'multiple property escapes in a single character class should be supported'
);
