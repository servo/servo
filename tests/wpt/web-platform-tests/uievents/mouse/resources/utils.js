// Sends to Window |w| the object |{type, param}|.
function sendMessage(w, type, param) {
  w.postMessage({"type": type, "param": param}, "*");
}

// Returns a |Promise| that gets resolved with the event object when |target|
// receives an event of type |event_type|.
function getEvent(event_type, target) {
  return new Promise(resolve => {
    target.addEventListener(event_type, e => resolve(e), {once: true});
  });
}

// Returns a |Promise| that gets resolved with |event.data| when |window|
// receives from |source| a "message" event whose |event.data.type| matches the string
// |message_data_type|.
function getMessageData(message_data_type, source) {
  return new Promise(resolve => {
    function waitAndRemove(e) {
      if (e.source != source || !e.data || e.data.type != message_data_type)
        return;
      window.removeEventListener("message", waitAndRemove);
      resolve(e.data);
    }
    window.addEventListener("message", waitAndRemove);
  });
}
