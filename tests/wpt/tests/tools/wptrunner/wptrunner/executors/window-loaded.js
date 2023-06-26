const [resolve] = arguments;

if (document.readyState != "complete") {
  window.addEventListener("load", () => {
    resolve();
  }, { once: true });
} else {
  resolve();
}
