// META: script=websocket.sub.js
// META: timeout=long

async_test(t => {
  window.wsurl = 'wss://' + __SERVER__NAME + ':' + __SECURE__PORT +
      '/does-not-exist';
  let wsframe;
  window.wsonerror = () => {
    wsframe.remove();
    // If this didn't crash then the test passed.
    t.done();
  };
  wsframe = document.createElement('iframe');
  wsframe.srcdoc = `<script>
const ws = new WebSocket(parent.wsurl);
ws.onerror = parent.wsonerror;
</script>`;
  onload = () => document.body.appendChild(wsframe);
}, 'removing an iframe from within an onerror handler should work');

done();
