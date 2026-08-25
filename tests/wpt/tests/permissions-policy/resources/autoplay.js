

function simulateGesture(t, callback) {
  // Get or create the target element.
  let target = document.getElementById('target');
  if (!target) {
    target = document.createElement('button');
    target.setAttribute('id', 'target');
    document.body.appendChild(target);
  }

  // Simulate a gesture in the top frame to remove any gesture based autoplay
  // restrictions.
  test_driver.click(target).then(callback, t.unreached_func('click failed'));
}

function isAutoplayAllowed() {
  return new Promise((resolve, reject) => {
    const video = document.createElement('video');
    video.src = getVideoURI('/media/A4');
    video.play().then(() => resolve(true), (e) => {
      if (e.name == 'NotAllowedError')
        resolve(false);
      else
        resolve(true);
    });
  });
}
