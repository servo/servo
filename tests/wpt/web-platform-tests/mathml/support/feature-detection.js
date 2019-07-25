// This is a helper for MathML feature detection.
// It is indented to be used to prevent false negative test results.

var MathMLFeatureDetection = {
    has_mspace: function() {
        if (!this.hasOwnProperty("_has_mspace")) {
            document.body.insertAdjacentHTML("beforeend", "<math><mspace width='20px'></mspace></math>");
            var math = document.body.lastElementChild;
            this._has_mspace =
                math.firstChild.getBoundingClientRect().width > 10;
            document.body.removeChild(math);
        }
        return this._has_mspace;
    }
};
