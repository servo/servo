(function() {
  busy_wait(60);
  Promise.resolve().then(busy_wait);
  new URLSearchParams([["a", "hello"]]).forEach((value, key) => {
    document.querySelector("#dummy").innerText += value;
  });
})();
