// META: script=resources/util.js

async_test((t) => {
  window.addEventListener('message', message_listener(t, "FAIL"));
  var iframe = document.createElement("iframe");
  iframe.src = ECHO_URL;
  document.body.appendChild(iframe);
}, "Critical-CH iframe");

async_test((t) => {
  window.addEventListener('message', message_listener(t, "FAIL"));
  var iframe = document.createElement("iframe");
  iframe.src = ECHO_URL+"?multiple=true";
  document.body.appendChild(iframe);
}, "Critical-CH w/ multiple headers and iframe");
