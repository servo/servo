// Copyright (C) 2019 Leo Balter. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: pending
description: >
    Hashbang comments should not require a newline afterwards
info: |
    HashbangComment::
      #! SingleLineCommentChars[opt]
features: [hashbang]
---*/

assert.sameValue(eval('#!'), undefined);
