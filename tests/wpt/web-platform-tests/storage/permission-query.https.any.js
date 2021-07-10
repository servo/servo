// META: title=The Permission API registration for "persistent-storage"

promise_test(async t => {
  const status =
        await navigator.permissions.query({name: 'persistent-storage'});
  assert_equals(status.constructor, PermissionStatus,
                'query() result should resolve to a PermissionStatus');
  assert_true(['granted','denied', 'prompt'].includes(status.state),
              'state should be a PermissionState');
}, 'The "persistent-storage" permission is recognized');
