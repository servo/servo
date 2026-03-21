// This js file actually calls fetch to load an image to
// an element.
async function load_image(label, img_element, image_to_load = "blue.png") {
  const url = "/images/" + image_to_load + "?" + label;
  const response = await fetch(url);
  blob = await response.blob();
  const imgURL = URL.createObjectURL(blob);
  img_element.src = imgURL;
}
