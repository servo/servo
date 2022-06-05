// META: script=/webcodecs/videoFrame-utils.js

promise_test(async t => {
  let imgElement = document.createElement('img');
  let loadPromise = new Promise(r => imgElement.onload = r);
  imgElement.src = 'data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAUAAAAFCAYAAACNbyblAAAAHElEQVQI12P4//8/w38GIAXDIBKE0DHxgljNBAAO9TXL0Y4OHwAAAABJRU5ErkJggg==';
  await loadPromise;
  verifyTimestampRequiredToConstructFrame(imgElement);
}, 'Test that timestamp is required when constructing VideoFrame from HTMLImageElement');

promise_test(async t => {
    let svgImageElement = document.createElementNS('http://www.w3.org/2000/svg','image');
    let loadPromise = new Promise(r => svgImageElement.onload = r);
    svgImageElement.setAttributeNS('http://www.w3.org/1999/xlink','href','data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAUAAAAFCAYAAACNbyblAAAAHElEQVQI12P4//8/w38GIAXDIBKE0DHxgljNBAAO9TXL0Y4OHwAAAABJRU5ErkJggg==');
    await loadPromise;
    verifyTimestampRequiredToConstructFrame(svgImageElement);
}, 'Test that timestamp is required when constructing VideoFrame from SVGImageElement');

promise_test(async t => {
  verifyTimestampRequiredToConstructFrame(document.createElement('canvas'))
}, 'Test that timeamp is required when constructing VideoFrame from HTMLCanvasElement');
