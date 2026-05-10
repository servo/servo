// Copyright 2023 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-initializecollator
description: resolved ignorePunctuation is the same as the one specified in option bag.
locale: [en, th, ja]
---*/
['en', 'th', 'ja'].forEach((locale) => {
  [true, false].forEach((ignorePunctuation) => {
    assert.sameValue(
      (new Intl.Collator(locale, {ignorePunctuation}))
          .resolvedOptions().ignorePunctuation,
      ignorePunctuation);
  });
});
