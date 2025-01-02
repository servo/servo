let frame;
window.unloadChild = function() {
  document.body.removeChild(frame);
};

promise_test(async t => {
  frame = document.createElement('iframe');
  frame.srcdoc = `<script>
    navigator.hid.getDevices();
    window.parent.unloadChild();
    </script>`;
  document.body.appendChild(frame);
}, 'Unload child iframe with pending getDevices promise');

promise_test(async t => {
  frame = document.createElement('iframe');
  frame.srcdoc = `<script>
    navigator.hid.requestDevice({filters: []});
    window.parent.unloadChild();
    </script>`;
  document.body.appendChild(frame);
}, 'Unload child iframe with pending requestDevice promise');
