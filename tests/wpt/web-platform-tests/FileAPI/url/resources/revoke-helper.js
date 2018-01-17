self.addEventListener('message', e => {
  URL.revokeObjectURL(e.data.url);
  self.postMessage('revoked');
});
