// These functions are placed in this file to ensure their source URLs are
// distinguishable from those of their callers, allowing for verification that
// the reported 'initiator URL' in resource timing is correct.
async function load_image(label, img_element, image_to_load = "blue.png") {
  const url = "/images/" + image_to_load + "?" + label;
  const response = await fetch(url);
  blob = await response.blob();
  const imgURL = URL.createObjectURL(blob);
  img_element.src = imgURL;
}

function fetch_in_function(resource) {
  fetch(resource);
}
