(function() {
  busy_wait(60);
  new URLSearchParams([["a", "hello"]]).forEach((value, key) => {
    document.querySelector("#dummy").innerText += value;
  });
  generate_loaf_now();
})();
