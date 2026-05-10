// Copyright 2018 André Bargull; Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

// Split from intl402/Locale/likely-subtags-grandfathered.js
/*---
esid: sec-intl.getcanonicallocales
description: >
    Verifies canonicalization of specific tags.
---*/


const regularGrandfathered = [
    {
        tag: "art-lojban",
        canonical: "jbo",
    },
    {
        tag: "zh-guoyu",
        canonical: "zh",
    },
    {
        tag: "zh-hakka",
        canonical: "hak",
    },
    {
        tag: "zh-xiang",
        canonical: "hsn",
    },
];

for (const {tag, canonical} of regularGrandfathered) {
    assert.sameValue(Intl.getCanonicalLocales(tag)[0], canonical);
}
