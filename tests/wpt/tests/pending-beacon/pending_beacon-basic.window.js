'use strict';

test(() => {
  assert_false(window.hasOwnProperty('PendingGetBeacon'));
}, `PendingGetBeacon is not supported in non-secure context.`);

test(() => {
  assert_false(window.hasOwnProperty('PendingPostBeacon'));
}, `PendingPostBeacon is not supported in non-secure context.`);
