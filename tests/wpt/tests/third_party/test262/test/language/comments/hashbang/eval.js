// Copyright (C) 2019 Leo Balter. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: pending
description: >
    Hashbang comments should be available in Script evaluator contexts. (direct eval)
info: |
    HashbangComment::
      #! SingleLineCommentChars[opt]
features: [hashbang]
---*/

assert.sameValue(eval('#!\n'), undefined);
assert.sameValue(eval('#!\n1'), 1)
assert.sameValue(eval('#!2\n'), undefined);
