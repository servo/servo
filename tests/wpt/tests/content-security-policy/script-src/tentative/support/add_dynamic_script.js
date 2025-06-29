var head = document.getElementsByTagName('head')[0];
var script = document.createElement('script');
script.type = 'text/javascript';
script.src = new URL("./externalScript.js", document.location).toString();
head.appendChild(script);
