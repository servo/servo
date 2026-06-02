"use strict";
setup({ single_test: true });

const xhr = new XMLHttpRequest();

// In jsdom's implementation, this would cause a crash, as after firing readystatechange for HEADERS_RECEIVED, it would
// try to manipulate internal state. But that internal state got cleared during abort(). So jsdom needed to be modified
// to check if that internal state had gone away as a result of firing readystatechange, and if so, bail out.

xhr.addEventListener("readystatechange", () => {
  if (xhr.readyState === xhr.HEADERS_RECEIVED) {
    xhr.abort();
  } else if (xhr.readyState === xhr.DONE) {
    done();
  }
});

xhr.open("GET", "/common/blank.html");
xhr.send();
