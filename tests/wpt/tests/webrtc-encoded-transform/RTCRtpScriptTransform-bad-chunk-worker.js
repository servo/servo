onrtctransform = async (event) => {
  const { port } = event.transformer.options;
  port.postMessage("started");

  const reader = event.transformer.readable.getReader();
  const writer = event.transformer.writable.getWriter();

  const { done, value } = await reader.read();

  writer.write(null).catch(err => port.postMessage([err.name, 'null']));
  writer.write(value).catch(err => port.postMessage([err.name, value.constructor.name]));
};
self.postMessage('registered');
