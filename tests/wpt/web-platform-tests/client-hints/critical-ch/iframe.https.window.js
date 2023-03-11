// META: script=resources/util.js

async_test((t) => {
  var iframe = document.createElement("iframe");
  iframe.src = ECHO_URL;
  document.body.appendChild(iframe);
  iframe.contentWindow.addEventListener('message', message_listener(t, "FAIL"));
}, "Critical-CH iframe");

async_test((t) => {
  var iframe = document.createElement("iframe");
  iframe.src = ECHO_URL+"?multiple=true";
  document.body.appendChild(iframe);
  iframe.contentWindow.addEventListener('message', message_listener(t, "FAIL"));
}, "Critical-CH w/ multiple headers and iframe");
