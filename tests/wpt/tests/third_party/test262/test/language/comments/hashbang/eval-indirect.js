// Copyright (C) 2019 Leo Balter. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: pending
description: >
    Hashbang comments should be available in Script evaluator contexts. (indirect eval)
info: |
    HashbangComment::
      #! SingleLineCommentChars[opt]
features: [hashbang]
---*/

assert.sameValue((0, eval)('#!\n'), undefined);
assert.sameValue((0, eval)('#!\n1'), 1)
assert.sameValue((0, eval)('#!2\n'), undefined);
