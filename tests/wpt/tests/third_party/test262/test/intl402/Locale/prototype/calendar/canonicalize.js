// Copyright 2024 Google Inc.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-makelocalerecord
description: Checks canonicalization of option.
info: |
    MakeLocaleRecord ( _tag_, _options_, _localeExtensionKeys_ )
    ...
    4.e. If _overrideValue is not *undefined*, then
        i. Set _value_ to CanonicalizeUValue(_key_, _overrideValue_).
    ...
      f. Set _result_.[[<_key_>]] to _value_.
features: [Intl.Locale]
---*/

const loc = new Intl.Locale('en', {calendar: 'islamicc'});

assert.sameValue(loc.toString(), "en-u-ca-islamic-civil",
    "'islamicc' should be canonicalized to 'islamic-civil'");

assert.sameValue(loc.calendar, "islamic-civil",
    "'islamicc' should be canonicalized to 'islamic-civil'");
