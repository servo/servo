/**
 * Utilities to be used with document_domain_frame.html.
 */

/**
 * Send a message to the frame and resolve a promise when a response is received.
 *
 * Supported messages:
 *
 * 1) { domain: something }.  Has the subframe try to set document.domain to the
 * given value, and message back 'Done' if that succeeds or an error name if it
 * fails.
 *
 * 2) 'poke-at-parent'.  Has the subframe try to synchronously attempt to access
 * the parent's DOM, read out a string value, and message it back to the parent.
 * Again, sends back the error name if that fails.
 *
 * 3) { 'poke-at-sibling': name }.  Has the subframe try to synchronously
 * attempt to access the DOM of the sibling with the given name, read out a
 * string value, and message it back to the parent.
 */
function postMessageToFrame(frame, message) {
  return new Promise(resolve => {
    var c = new MessageChannel();
    c.port1.onmessage = e => {
      resolve({ data: e.data, frame: frame })
    };
    frame.contentWindow.postMessage(message, '*', [c.port2]);
  });
}

/**
 * Create a frame that loads document_domain_frame.html and resolves a promise
 * when the frame is loaded enough to be sending and receiving messages.
 *
 * If a "name" argument is provided, that name is used for the iframe, so
 *
 * If a "hostname" argument is provided, that hostname is used for the load, to
 * allow testing details of the behavior when different sorts of hostnames are
 * used.
 */
function createFrame(t, name, hostname) {
  return new Promise(resolve => {
    var i = document.createElement('iframe');
    if (hostname) {
      i.src = `//${hostname}:{{location[port]}}/html/browsers/origin/relaxing-the-same-origin-restriction/support/document_domain_frame.html`;
    } else {
      i.src = "support/document_domain_frame.html";
    }
    if (name) {
      i.name = name;
    }
    var listener = m => {
      if (m.source == i.contentWindow)
        resolve(i);
    }
    window.addEventListener('message', listener);
    t.add_cleanup(() => {
      i.remove();
      window.removeEventListener('message', listener);
    });
    document.body.appendChild(i);
  });
}

