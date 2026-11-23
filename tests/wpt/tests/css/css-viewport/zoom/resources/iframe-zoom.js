function nextFrame() {
  return new Promise(resolve => requestAnimationFrame(() => requestAnimationFrame(resolve)));
}

async function loadIframe({ srcdoc, style = "", container = document.body, test }) {
  const iframe = document.createElement("iframe");
  iframe.style.cssText = style;
  iframe.srcdoc = srcdoc;
  test?.add_cleanup(() => iframe.remove());
  const loaded = new Promise(resolve => iframe.addEventListener("load", resolve, { once: true }));
  container.appendChild(iframe);
  await loaded;
  return iframe;
}
