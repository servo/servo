var elem = document.documentElement.firstChild;
debug(elem.tagName);
debug(elem instanceof HTMLImageElement);
debug(elem.width);
elem.width = 1000;
debug(elem.width);  
