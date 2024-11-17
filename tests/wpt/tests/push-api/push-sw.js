async function postAll(data) {
  const clients = await self.clients.matchAll({ includeUncontrolled: true });
  for (const client of clients) {
    client.postMessage(data);
  }
}

onpush = async ev => {
  postAll({
    constructor: ev.constructor.name,
    data: ev.data ? new Uint8Array(await ev.data.arrayBuffer()) : null, // .bytes() is Firefox only as of 2024-10
  })
}
