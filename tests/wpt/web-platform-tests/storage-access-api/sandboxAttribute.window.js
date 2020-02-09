'use strict';

test(() => {
  let iframe = document.createElement('iframe');
  assert_true(iframe.sandbox.supports('allow-storage-access-by-user-activation'), '`allow-storage-access-by-user-activation`' +
    'sandbox attribute should be supported');
}, "`allow-storage-access-by-user-activation` sandbox attribute is supported");
