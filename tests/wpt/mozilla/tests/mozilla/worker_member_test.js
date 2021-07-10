let member = location.search.slice(1);
var binding = new TestBinding();
postMessage(binding[member]);