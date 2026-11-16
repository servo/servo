"use strict";
/* global token */
/* exported controlledResponse, requestResource */

async function requestResource(url) {
  const xhr = new XMLHttpRequest();
  await new Promise((resolve, reject) => {
    xhr.onload = resolve;
    xhr.onerror = reject;
    xhr.open("GET", url);
    xhr.send();
  });
  assert_equals(xhr.status, 200, `Loading ${url}`);
  return xhr.responseText;
}

function controlledResponse(t, body, type) {
  const url = new URL("/html/semantics/resources/controlled-response.py", location);
  url.search = new URLSearchParams({ key: token(), body, type });
  const releaseURL = new URL(url);
  releaseURL.searchParams.set("action", "release");
  let released;
  function release() {
    released ||= requestResource(releaseURL);
    return released;
  }
  t.add_cleanup(release);
  return { url: url.href, release };
}
