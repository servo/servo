// Copyright 2025 Richard Gibson. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
author: Richard Gibson
description: >
  Unicode property "Script_Extensions" and alias "scx" must support
  special value "Unknown" and alias "Zzzz" (cf.
  https://www.unicode.org/reports/tr24/#Script_Extensions_Def).
esid: sec-compiletocharset
features: [regexp-unicode-property-escapes]
---*/

/\p{Script_Extensions=Unknown}/u;
/\p{Script_Extensions=Zzzz}/u;
/\p{scx=Unknown}/u;
/\p{scx=Zzzz}/u;
