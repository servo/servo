// Copyright 2023 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-initializecollator
description: resolved ignorePunctuation come from locale default instead of false.
locale: [en, th, ja]
---*/
assert.sameValue(
  (new Intl.Collator("en")).resolvedOptions().ignorePunctuation,
  false, "English default ignorePunctuation to false");
assert.sameValue(
  (new Intl.Collator("th")).resolvedOptions().ignorePunctuation,
  true, "Thai default ignorePunctuation to true");
assert.sameValue(
  (new Intl.Collator("ja")).resolvedOptions().ignorePunctuation,
  false, "Japanese default ignorePunctuation to false");
