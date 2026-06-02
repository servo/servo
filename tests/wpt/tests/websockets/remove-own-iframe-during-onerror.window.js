// META: script=constants.sub.js
// META: timeout=long
// META: variant=?default
// META: variant=?wss
// META: variant=?wpt_flags=h2

async_test(t => {
  window.wsurl = SCHEME_DOMAIN_PORT + '/does-not-exist';
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
