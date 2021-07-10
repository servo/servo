test(() => {
  assert_true('GLOBAL' in self);
}, 'GLOBAL exists');

scripts.push('expect-global.js');
