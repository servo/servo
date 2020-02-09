// This creates a serialized <script> element that is useful for blob/data/srcdoc-style tests.

function createScript(sameOrigin, crossOrigin, parent="parent", id="") {
  return `<script>
const data = { id: "${id}",
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

Promise.all(records).then(() => window.${parent}.postMessage(data, "*"));
<\/script>`;
}
