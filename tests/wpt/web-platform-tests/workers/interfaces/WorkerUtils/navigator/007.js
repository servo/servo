var log = [];
var neverEncounteredValue = "This is not the value you are looking for.";
for (x in navigator) {
  // this should silently fail and not throw per webidl
  navigator[x] = neverEncounteredValue;
  if (navigator[x] === neverEncounteredValue)
    log.push(x);
}
postMessage(log.join(', '));