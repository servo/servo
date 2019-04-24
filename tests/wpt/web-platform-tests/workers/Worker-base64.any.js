// META: global=!default,worker
test(() => {
  assert_true(typeof atob === 'function');
  assert_true(typeof btoa === 'function');
}, 'Tests that atob() / btoa() functions are exposed to workers');
