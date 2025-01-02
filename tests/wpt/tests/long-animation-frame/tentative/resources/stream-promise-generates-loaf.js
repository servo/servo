(async() => {
  const response = await fetch("/common/dummy.xml");
  const {readable, writable} = new TransformStream({
    start() {},
    transform() {
      generate_loaf_now();
    }
  });
  response.body.pipeTo(writable);
  await readable.getReader().read();
})();
