onmessage = (e) => {
  const sab = e.data;
  const ta = new Int32Array(sab);
  Atomics.notify(ta, 0);
};
