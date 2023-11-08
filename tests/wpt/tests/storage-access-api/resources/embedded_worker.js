"use strict";

self.onmessage = async (message) => {
  function reply(data) {
    self.postMessage({data});
  }

  switch (message.data.command) {
    case "fetch": {
      const response = await fetch(message.data.url, {mode: 'cors', credentials: 'include'})
        .then((resp) => resp.text());
      reply(response);
      break;
    }
    default:
  }
};
