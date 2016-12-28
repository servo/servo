var c;

function handler(e, reply) {
  if (e.data.ping) {
    c.postMessage(e.data.ping);
    return;
  }
  if (e.data.blob) {
    c.postMessage({blob: new Blob(e.data.blob)});
  }
  c = new BroadcastChannel(e.data.channel);
  let messages = [];
  c.onmessage = e => {
      messages.push(e.data);
      if (e.data == 'done')
        reply(messages);
    };
  c.postMessage('from worker');
}

onmessage = e => handler(e, postMessage);

onconnect = e => {
  let port = e.ports[0];
  port.onmessage = e => handler(e, msg => port.postMessage(msg));
};
