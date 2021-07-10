// Helper to access the element, its associated loading promise, and also to
// resolve the promise.
class ElementLoadPromise {
  constructor(element_id) {
    this.element_id = element_id;
    this.promise = new Promise((resolve, reject) => {
      this.resolve = resolve
      this.reject = reject
    });
  }
  element() {
    return document.getElementById(this.element_id);
  }
}

// Returns if the image is complete and the lazily loaded image matches the expected image.
function is_image_fully_loaded(image, expected_image) {
  if (!image.complete || !expected_image.complete) {
    return false;
  }

  if (image.width != expected_image.width ||
      image.height != expected_image.height) {
    return false;
  }

  let canvas = document.createElement('canvas');
  canvas.width = image.width;
  canvas.height = image.height;
  let canvasContext = canvas.getContext("2d");
  canvasContext.save();
  canvasContext.drawImage(image, 0, 0);
  let data = canvasContext.getImageData(0, 0, canvas.width, canvas.height).data;

  canvasContext.restore();
  canvasContext.drawImage(expected_image, 0, 0);
  let expected_data = canvasContext.getImageData(0, 0, canvas.width, canvas.height).data;

  for (var i = 0; i < data.length; i++) {
    if (data[i] != expected_data[i]) {
      return false;
    }
  }
  return true;
}
