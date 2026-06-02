let messages = [];
const channel = new BroadcastChannel('foo');  // Access shared channel

channel.addEventListener('message', event => {
  messages.push(event.data);
});

function waitForEventsPromise(count) {
  return new Promise(resolve => {
    function checkMessages() {
      if (messages.length >= count) {
        channel.removeEventListener('message', checkMessages);  // Cleanup
        resolve(messages.length);
      }
    }
    checkMessages();
    channel.addEventListener('message', checkMessages);
  });
}