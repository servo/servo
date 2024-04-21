"use strict";

test_driver.set_test_context(window.top);

function waitForMessage(timestamp) {
  return new Promise(resolve => {
    const listener = (event) => {
      if (!timestamp || event.data.timestamp == timestamp) {
        window.removeEventListener("message", listener);
        resolve(event.data);
      }
    };
    window.addEventListener("message", listener);
  });
}

var iframe = document.createElement('iframe');
const queryString = window.location.search;
const urlParams = new URLSearchParams(queryString);
iframe.src = urlParams.get("inner_url");
document.body.appendChild(iframe);

window.addEventListener("message", async (event) => {
  function replyToParent(data) {
    parent.postMessage(
        {timestamp: event.data.timestamp, data}, "*");
  }

  if (!event.data["command"]) {
    return;
  }

  switch (event.data["command"]) {
    case "navigate_child":
      iframe.onload = () => replyToParent(event.data.url);
      iframe.src = event.data.url;
      break;
    case "reload":
    case "navigate":
      iframe.contentWindow.postMessage({timestamp, ...event.data}, "*");
      break;
    default:{
      const timestamp = event.data.timestamp;
      const p = waitForMessage(timestamp);
      iframe.contentWindow.postMessage({timestamp, ...event.data}, "*");
      replyToParent(await p.then(resp => resp.data));
      break;
    }
  }
});
