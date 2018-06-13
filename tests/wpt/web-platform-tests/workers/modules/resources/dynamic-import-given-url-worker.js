// This worker dynamically imports the script URL sent by postMessage(), and
// sends back an error name if the dynamic import fails.
self.addEventListener('message', msg_event => {
  import(msg_event.data).catch(e => postMessage(e.name));
});
