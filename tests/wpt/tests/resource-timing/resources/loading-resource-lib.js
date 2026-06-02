// This js file actually calls fetch to load an image to
// an element.
async function load_image(label, img_element) {
  const url = "/images/blue.png?"+label;
  const response = await fetch(url);
  blob = await response.blob();
  const imgURL = URL.createObjectURL(blob);
  img_element.src = imgURL;
}
