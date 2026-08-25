"use strict";

self.onmessage = ({ data }) => {
  // Write into the shared backing store so the sender can verify shared semantics.
  new Int32Array(data.buffer)[1] = 200;
  self.postMessage(data);
};
