function ScreenConsole(element) {
  this._element = element;
}

ScreenConsole.prototype.log = function () {
  var text = "";
  for (var i = 0; i < arguments.length; i++) {
    text += arguments[i] + " ";
  }
  console.log(text);
  this._element.innerText += text + "\n";
};

ScreenConsole.prototype.clear = function () {
  this._element.innerText = "";
};
