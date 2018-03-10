window.top.postMessage({
  "cookies": document.cookie,
  "expectation": document.querySelector('#data').innerText
}, "*");
