// This creates a serialized <script> element that is useful for blob/data/srcdoc-style tests.

function createScript(sameOrigin, crossOrigin, type="parent", id="") {
  return `<script>
const data = { id: "${id}",
               opener: !!window.opener,
               origin: window.origin,
               sameOriginNoCORPSuccess: false,
               crossOriginNoCORPFailure: false };
function record(promise, token, expectation) {
  return promise.then(() => data[token] = expectation, () => data[token] = !expectation);
}

const records = [
  record(fetch("${crossOrigin}/common/blank.html", { mode: "no-cors" }), "crossOriginNoCORPFailure", false)
];

if ("${sameOrigin}" !== "null") {
  records.push(record(fetch("${sameOrigin}/common/blank.html", { mode: "no-cors" }), "sameOriginNoCORPSuccess", true));
}

Promise.all(records).then(() => {
  // Using BroadcastChannel is useful for blob: URLs, which are always same-origin
  if ("${type}" === "channel") {
    const bc = new BroadcastChannel("${id}");
    bc.postMessage(data);
  } else {
    window.${type}.postMessage(data, "*");
  }
});
<\/script>`;
}
