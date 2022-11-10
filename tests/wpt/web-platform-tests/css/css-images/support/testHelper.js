function paintCanvases() {
  for (let canvas of document.getElementsByTagName("canvas")) {
    canvas.width = 50;
    canvas.height = 100;

    let ctx = canvas.getContext("2d");
    ctx.fillStyle = 'blue';
    ctx.fillRect(0, 0, 25, 50);

    ctx.fillStyle = 'green';
    ctx.fillRect(25, 0, 25, 50);

    ctx.fillStyle = 'red';
    ctx.fillRect(0, 50, 25, 50);

    ctx.fillStyle = 'yellow';
    ctx.fillRect(25, 50, 50, 50);
  }
}

function populateElements(imageSource) {
  let images = document.getElementsByTagName("img");
  for (var i = 0; i < images.length; i++)
    images[i].src = imageSource;

  paintCanvases();

  for (let video of document.getElementsByTagName("video"))
    video.poster = "support/exif-orientation-6-ru.jpg";
}
