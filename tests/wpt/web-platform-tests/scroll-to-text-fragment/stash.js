// Put test results into Stash
function stashResults(key, results) {
  fetch(`/scroll-to-text-fragment/stash.py?key=${key}`, {
    method: 'POST',
    body: JSON.stringify(results)
  });
}

// Fetch test results from the Stash
function fetchResults(key, resolve, reject) {
  fetch(`/scroll-to-text-fragment/stash.py?key=${key}`).then(response => {
    return response.text();
  }).then(text => {
    if (text) {
      try {
        const results = JSON.parse(text);
        resolve(results);
      } catch(e) {
        reject();
      }
    }
  });
}
