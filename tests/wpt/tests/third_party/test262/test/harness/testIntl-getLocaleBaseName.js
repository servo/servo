// Copyright (c) 2026 Igalia, S.L..  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: >
  The `getLocaleBaseName` method removes any extension subtags that may be
  present and leaves all other aspects of the language tag unmodified.
includes: [testIntl.js]
---*/

assert.sameValue(getLocaleBaseName('fr'), 'fr', 'language');
assert.sameValue(getLocaleBaseName('zh-Hans'), 'zh-Hans', 'language, script');
assert.sameValue(getLocaleBaseName('en-us'), 'en-us', 'language, region');
assert.sameValue(getLocaleBaseName('ru-Cyrl-BY'), 'ru-Cyrl-BY', 'language, script, region');

assert.sameValue(getLocaleBaseName('fr-u-co-phonebk'), 'fr', 'language, extension');
assert.sameValue(getLocaleBaseName('zh-Hans-u-co-phonebk'), 'zh-Hans', 'language, script, extension');
assert.sameValue(getLocaleBaseName('en-us-u-co-phonebk'), 'en-us', 'language, region, extension');
assert.sameValue(getLocaleBaseName('ru-Cyrl-BY-u-co-phonebk'), 'ru-Cyrl-BY', 'language, script, region, extension');
