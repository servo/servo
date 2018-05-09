// Dynamically import the script URL sent by postMessage().
self.addEventListener('message', e => {
  import(e.data);
});
