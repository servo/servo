// META: script=resources/multiple-content-disposition.js

runMultipleContentDispositionTests("HTTP/2", values => {
  const params = new URLSearchParams();
  for (const value of values) {
    params.append("value", value);
  }
  return `resources/multiple-content-disposition.h2.py?${params}`;
});
