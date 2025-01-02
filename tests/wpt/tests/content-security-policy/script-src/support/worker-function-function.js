var fn = function() {
    postMessage('Function() function blocked');
}
try {
    fn = new Function("", "postMessage('Function() function allowed');");
} catch (e) {}
fn();
