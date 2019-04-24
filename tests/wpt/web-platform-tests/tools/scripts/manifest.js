// grab test metadata from a test file
function __result_handler() {

    function __get_metadata() {
        var obj = new Object();
        var author = [];
        var assert = [];
        var help = [];
        var match = [];
        var mismatch = [];
        var flags = [];
        var nodes;

        nodes = document.querySelectorAll('link[rel="author"]');
        for (var i = 0; i < nodes.length; i++) {
            var href = nodes[i].getAttribute("href");
            var title = nodes[i].getAttribute("title");
            var s = title;
            if (href != null) {
                s += " <" + href + ">";
            }
            author.push(s);
        }
        if (nodes.length > 0) obj.author = author;
        nodes = document.querySelectorAll('meta[name="assert"]');
        for (var i = 0; i < nodes.length; i++) {
            assert.push(nodes[i].getAttribute("content"));
        }
        if (nodes.length > 0) obj.assert = assert;
        nodes = document.querySelectorAll('link[rel="help"]');
        for (var i = 0; i < nodes.length; i++) {
            help.push(nodes[i].getAttribute("href"));
        }
        if (nodes.length > 0) obj.help = help;
        nodes = document.querySelectorAll('link[rel="match"]');
        for (var i = 0; i < nodes.length; i++) {
            match.push(nodes[i].getAttribute("href"));
        }
        if (nodes.length > 0) obj.match = match;
        nodes = document.querySelectorAll('link[rel="mismatch"]');
        for (var i = 0; i < nodes.length; i++) {
            mismatch.push(nodes[i].getAttribute("href"));
        }
        if (nodes.length > 0) obj.match = mismatch;
        nodes = document.querySelectorAll('meta[name="flags"]');
        for (var i = 0; i < nodes.length; i++) {
            flags.push(nodes[i].getAttribute("content"));
        }
        if (nodes.length > 0) obj.flags = flags;

        return obj;
    }

    var meta = __get_metadata();
    var nodes;

    function copy(obj, prop, arr) {
        if (typeof arr !== "undefined") {
            var a = [];
            for (var i = 0; i<arr.length;i++) {
                a[i] = arr[i];
            }
            obj[prop] = a;
        }
    }


    var ret = new Object();
    ret.location = document.location.href;
    ret.type  = "manual";
    ret.tests = new Object();

    var node = document.querySelector('script[src$="/resources/testharness.js"]');
    if (node !== null) {
        ret.type = "script";
    }

    if (ret.type === "script") {
        if (typeof metadata_generator === "undefined"
            || Object.keys(metadata_generator.currentMetadata).length === 0)
            return "WRAPPER:TRY_AGAIN";
        else {
            for (var key in metadata_generator.currentMetadata) {
                var obj = metadata_generator.currentMetadata[key];
                var newtest = new Object();
                ret.tests[key]= newtest;
                if (typeof obj.help === "undefined") {
                    copy(newtest, "help", meta.help);
                } else if (typeof obj.help === "string") {
                    newtest.help = [ obj.help ];
                }
                if (typeof obj.author === "undefined") {
                    copy(newtest, "author", meta.author);
                } else if (typeof obj.author === "string") {
                    newtest.author = [ obj.author ];
                }
                if (typeof obj.assert === "undefined") {
                    copy(newtest, "assert", meta.assert);
                } else if (typeof obj.assert === "string") {
                    newtest.assert = [ obj.assert ];
                }
                copy(newtest, "match", meta.match);
                copy(newtest, "mismatch", meta.mismatch);
                copy(newtest, "flags", meta.flags);
            }
            return ret;
        }
    } else {
        var newtest = meta;
        ret.tests[document.title]= newtest;

        if (typeof newtest.match !== "undefined"
            || typeof newtest.mismatch !== "undefined") {
            ret.type = "reftest";
        }

        return ret;
    }

}

function __give_up() {
    var ret = new Object();
    ret.location = document.location.href;
    ret.type  = "manual";
    ret.tests = new Object();

    var node = document.querySelector('script[src$="/resources/testharness.js"]');
    if (node !== null) {
        ret.type = "script";
    } else if (typeof newtest.match !== "undefined"
               || typeof newtest.mismatch !== "undefined") {
        ret.type = "reftest";
    }

    var newtest = __get_metadata();
    ret.tests[document.title]= newtest;

    return ret;
}
