(function() {
  busy_wait(60);
  Promise.resolve().then(() => window.generate_loaf_now());
  new URLSearchParams([["a", "hello"]]).forEach((value, key) => {
    document.querySelector("#dummy").innerText += value;
  });
})();
