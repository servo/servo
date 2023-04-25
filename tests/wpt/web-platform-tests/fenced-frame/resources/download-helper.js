function StreamDownloadFinishDelay() {
  return 1000;
}

function DownloadVerifyDelay() {
  return 1000;
}

async function VerifyDownload(test_obj, token) {
  const verifyToken = async (token) => {
    const url = `resources/download-stash.py?verify-token&token=${token}`;
    const response = await fetch(url);
    if (!response.ok) {
      throw new Error('An error happened in the server');
    }
    const message = await response.text();
    return message === 'TOKEN_SET';
  };

  return new Promise((resolve) => {
    test_obj.step_wait(
        async () => {
          const result = await verifyToken(token);
          resolve(result);
        },
        'Check if the download has finished or not',
        StreamDownloadFinishDelay() + DownloadVerifyDelay());
  });
}
