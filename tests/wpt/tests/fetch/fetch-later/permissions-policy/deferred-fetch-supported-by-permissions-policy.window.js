// META: title=The feature list should advertise deferred-fetch
'use strict';

// https://w3c.github.io/webappsec-permissions-policy/#dom-permissions-policy-features
// https://wicg.github.io/local-fonts/#permissions-policy
test(() => {
  assert_in_array('deferred-fetch', document.permissionsPolicy.features());
  assert_in_array('deferred-fetch-minimal', document.permissionsPolicy.features());
}, 'document.permissionsPolicy.features should advertise deferred-fetch.');
