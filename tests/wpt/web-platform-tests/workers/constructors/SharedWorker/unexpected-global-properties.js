var unexpected = 'open print stop getComputedStyle getSelection releaseEvents captureEvents alert confirm prompt addEventStream removeEventStream back forward attachEvent detachEvent navigate DOMParser XMLSerializer XPathEvaluator XSLTProcessor opera Image Option frames Audio SVGUnitTypes SVGZoomAndPan java netscape sun Packages ByteArray closed defaultStatus document event frameElement history innerHeight innerWidth opener outerHeight outerWidth pageXOffset pageYOffset parent screen screenLeft screenTop screenX screenY status top window length'.split(' '); // iterated window in opera and removed expected ones
var log = '';
for (var i = 0; i < unexpected.length; ++i) {
  if (unexpected[i] in self)
    log += unexpected[i] + ' ';
}
onconnect = function(e) {
  e.ports[0].postMessage(log);
};