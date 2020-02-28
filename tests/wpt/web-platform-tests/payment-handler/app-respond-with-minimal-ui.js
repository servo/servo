self.addEventListener('canmakepayment', event => {
  event.respondWithMinimalUI(event.methodData[0].data.test);
});
