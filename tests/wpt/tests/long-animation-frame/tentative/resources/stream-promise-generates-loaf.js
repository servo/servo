(async() => {
  const response = await fetch("/common/dummy.xml");
  const {readable, writable} = new TransformStream({
    start() {},
    transform() {
      const deadline = performance.now() + 360;
      while (performance.now() < deadline) {}
    }
  });
  response.body.pipeTo(writable);
  await readable.getReader().read();
})();
