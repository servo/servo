test(() => {
  const buffer = new WebAssembly.Memory({initial: 4}).buffer;
  assert_throws(new TypeError(), () => {
    postMessage('foo', '*', [buffer]);
  });
});
