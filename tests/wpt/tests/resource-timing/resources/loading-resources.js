//Fetching the Stylesheet
var link = document.createElement("link");
link.rel = "stylesheet";
link.href = "resources/empty_style.css?no_cache";
link.id = "link_id";
document.head.appendChild(link);

// Fetching an image
var img = document.createElement("img");
img.src = "/images/blue.png?no_cache";
img.alt = "Sample Image for testing initiator Attribute";
img.id = "img_id"
document.body.appendChild(img);

// Inserting a script element
var script = document.createElement("script");
script.src = "resources/empty.js?no_cache";
script.id = "script_id"
document.body.appendChild(script);

//Inserting a html document in an iframe
var iframe = document.createElement("iframe");
iframe.src = "resources/green.html?no_cache";
iframe.id = "iframe_id";
document.body.appendChild(iframe);
