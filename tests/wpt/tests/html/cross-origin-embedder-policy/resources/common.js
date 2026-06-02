async function createIsolatedFrame(origin, headers) {
  const parent = document.createElement('iframe');
  const parent_loaded = new Promise(r => parent.onload = () => { r(parent); });
  const error = new Promise(r => parent.onerror = r);
  parent.src = origin + "/common/blank.html?pipe=" + headers;
  parent.anonymous = false;
  document.body.appendChild(parent);
  return [parent_loaded, error];
}

async function IsCrossOriginIsolated(from_token) {
  const reply_token = token();
  send(from_token, `
    send("${reply_token}", self.crossOriginIsolated);
  `);
  const reply = await receive(reply_token);
  assert_true(reply.match(/true|false/) != null);
  return reply == 'true';
}