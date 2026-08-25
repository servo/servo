// META: title=SpeechRecognition in a detached iframe test

test(() => {
  // Create the iframe and append it to the document.
  const iframe = document.createElement("iframe");
  document.body.appendChild(iframe);
  const frameWindow = iframe.contentWindow;

  // Detach the iframe.
  iframe.remove();

  assert_equals(
    undefined,
    frameWindow.SpeechRecognition || frameWindow.webkitSpeechRecognition,
  );
}, "SpeechRecognition constructor does not exist in detached iframes");

test((t) => {
  // Create the iframe and append it to the document.
  const iframe = document.createElement("iframe");
  document.body.appendChild(iframe);
  const frameWindow = iframe.contentWindow;
  const frameDOMException = frameWindow.DOMException;

  frameWindow.SpeechRecognition =
    frameWindow.SpeechRecognition || frameWindow.webkitSpeechRecognition;
  const speechRecognition = new frameWindow.SpeechRecognition();

  // Detach the iframe.
  iframe.remove();

  assert_throws_dom("InvalidStateError", frameDOMException, () =>
    speechRecognition.start(),
  );
}, "SpeechRecognition.start() on detached frame throws");
