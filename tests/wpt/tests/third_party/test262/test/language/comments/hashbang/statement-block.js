// Copyright (C) 2019 Leo Balter. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: pending
description: >
    Hashbang comments should only be allowed at the start of source texts and should not be allowed within blocks.
info: |
    HashbangComment::
      #! SingleLineCommentChars[opt]
negative:
  phase: parse
  type: SyntaxError
features: [hashbang]
---*/

$DONOTEVALUATE();

{
  #!
}
