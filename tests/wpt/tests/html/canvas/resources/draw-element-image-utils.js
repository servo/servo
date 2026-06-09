function resizeToPixelGrid(canvas) {
  return new Promise(resolve => {
    new ResizeObserver(entries => {
      canvas.width = entries[0].devicePixelContentBoxSize[0].inlineSize;
      canvas.height = entries[0].devicePixelContentBoxSize[0].blockSize;
      setTimeout(resolve);
    }).observe(canvas);
  });
}

function computeScaledDestinationSize(cvs, target, scaleX, scaleY, outsetWidth, outsetHeight) {
  let targetWidth, targetHeight;
  outsetWidth = outsetWidth || 0;
  outsetHeight = outsetHeight || 0;

  if (target instanceof Element) {
    if (getComputedStyle(target).boxSizing != "border-box") {
      throw new TypeError("'box-sizing:border-box' is required to compute" +
                          " accurate destination size.");
    }
    const canvasScaleX = cvs.width / cvs.clientWidth;
    const canvasScaleY = cvs.height / cvs.clientHeight;
    const style = getComputedStyle(target);
    targetWidth = canvasScaleX * (Number.parseFloat(style.width) + outsetWidth);
    targetHeight = canvasScaleY * (Number.parseFloat(style.height) + outsetHeight);
  }

  if (target instanceof ImageData) {
    targetWidth = target.width;
    targetHeight = target.height;
  }

  return [Math.ceil(targetWidth * scaleX), Math.ceil(targetHeight * scaleY)];
}

function computeExplicitDestinationSize(cvs, scaleX, scaleY, swidth, sheight) {
  const targetWidth = scaleX * swidth;
  const targetHeight = scaleY * sheight;

  // Scale factor from CSS pixels to canvas grid.
  const canvasScaleX = cvs.width / cvs.clientWidth;
  const canvasScaleY = cvs.height / cvs.clientHeight;

  // Destination size in canvas grid
  const destWidth = Math.ceil(targetWidth * canvasScaleX);
  const destHeight = Math.ceil(targetHeight * canvasScaleY);

  return [destWidth, destHeight];
}


export { resizeToPixelGrid,
         computeScaledDestinationSize,
         computeExplicitDestinationSize };
