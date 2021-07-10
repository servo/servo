test(() => {
  const buffer = new WebAssembly.Memory({initial: 4}).buffer;
  assert_throws_js(TypeError, () => {
    postMessage('foo', '*', [buffer]);
  });
});
