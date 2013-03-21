//window.alert(ClientRect);
//window.alert(ClientRectList);

window.alert("1");
let elem = document.documentElement;
window.alert(elem);
window.alert("2");
var rects = elem.getClientRects();
window.alert("3");
window.alert(rects);
window.alert(rects.length);
window.alert("4");
let rect = rects[0];
window.alert(rect);
/*window.alert(Object.prototype.toString.call(rect.__proto__));
window.alert(rect.__proto__ === Object.getPrototypeOf(rect));
window.alert(rect.__proto__.top);
window.alert(Object.getPrototypeOf(rect).top);*/
window.alert(rect.top);
window.alert(rect.bottom);
window.alert(rect.left);
window.alert(rect.right);
window.alert(rect.width);
window.alert(rect.height);

window.alert("HTMLCollection:");
let tags = document.getElementsByTagName("head");
//let tag = tags[0];
window.alert(tags);
window.alert(tags.length);
window.alert(tags[0]);
window.alert(tags[1]);
window.alert(tags[2]);
window.alert(tags[3]);
