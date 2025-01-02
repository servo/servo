// META: title=The feature list should advertise deferred-fetch
'use strict';

// https://w3c.github.io/webappsec-permissions-policy/#dom-permissions-policy-features
// https://wicg.github.io/local-fonts/#permissions-policy
test(() => {
  assert_in_array('deferred-fetch', document.featurePolicy.features());
}, 'document.featurePolicy.features should advertise deferred-fetch.');
