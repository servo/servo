function throw_function() {
  throw new Error("an error");
}

function load_image() {
  let img = document.createElement('img');
  document.body.append(img);
  img.src = "/xhr/resources/img.jpg"
}
