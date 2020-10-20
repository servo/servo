'use strict';

// Creates a new iframe in |doc|, calls |func| on it and appends it as a child
// of |doc|.
// Returns a promise that resolves to the iframe once loaded (successfully or
// not).
// The iframe is removed from |doc| once test |t| is done running.
//
// NOTE: Because iframe elements always invoke the onload event handler, even
// in case of error, we cannot wire onerror to a promise rejection. The Promise
// constructor requires users to resolve XOR reject the promise.
function append_child_frame_with(t, doc, func) {
  return new Promise(resolve => {
      const child = doc.createElement("iframe");
      func(child);
      child.onload = () => { resolve(child); };
      doc.body.appendChild(child);
      t.add_cleanup(() => { doc.body.removeChild(child); });
    });
}

// Appends a child iframe to |doc| sourced from |src|.
//
// See append_child_frame_with() for more details.
function append_child_frame(t, doc, src) {
  return append_child_frame_with(t, doc, child => { child.src = src; });
}
