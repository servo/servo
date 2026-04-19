// Copyright 2018 André Bargull; Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-intl.locale
description: >
    Verifies canonicalization of specific tags.
info: |
    ApplyOptionsToTag( tag, options )
    10. Return CanonicalizeLanguageTag(tag).
features: [Intl.Locale]
---*/

const validLanguageTags = {
    // Duplicate keywords are removed.
    "da-u-ca-gregory-ca-buddhist": "da-u-ca-gregory",

    // Keywords currently used in Intl specs are reordered in US-ASCII order.
    "zh-u-nu-hans-ca-chinese": "zh-u-ca-chinese-nu-hans",
    "zh-u-ca-chinese-nu-hans": "zh-u-ca-chinese-nu-hans",

    // Even keywords currently not used in Intl specs are reordered in US-ASCII order.
    "de-u-cu-eur-nu-latn": "de-u-cu-eur-nu-latn",
    "de-u-nu-latn-cu-eur": "de-u-cu-eur-nu-latn",

    // Attributes in Unicode extensions are reordered in US-ASCII order.
    "pt-u-attr-ca-gregory": "pt-u-attr-ca-gregory",
    "pt-u-attr1-attr2-ca-gregory": "pt-u-attr1-attr2-ca-gregory",
    "pt-u-attr2-attr1-ca-gregory": "pt-u-attr1-attr2-ca-gregory",
};

for (const [langtag, canonical] of Object.entries(validLanguageTags)) {
    assert.sameValue(
      new Intl.Locale(langtag).toString(),
      canonical,
      `new Intl.Locale("${langtag}").toString() returns "${canonical}"`
    );
}
