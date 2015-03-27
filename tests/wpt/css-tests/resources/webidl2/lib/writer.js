
(function () {

    var write = function (ast, opt) {
        var curPea = ""
        ,   curTPea = ""
        ,   opt = opt || {}
        ,   noop = function (str) { return str; }
        ,   optNames = "type".split(" ")
        ,   context = []
        ;
        for (var i = 0, n = optNames.length; i < n; i++) {
            var o = optNames[i];
            if (!opt[o]) opt[o] = noop;
        }
        
        var literal = function (it) {
            return it.value;
        };
        var wsPea = function (it) {
            curPea += it.value;
            return "";
        };
        var wsTPea = function (it) {
            curTPea += it.value;
            return "";
        };
        var lineComment = function (it) {
            return "//" + it.value + "\n";
        };
        var multilineComment = function (it) {
            return "/*" + it.value + "*/";
        };
        var type = function (it) {
            if (typeof it === "string") return opt.type(it); // XXX should maintain some context
            if (it.union) return "(" + it.idlType.map(type).join(" or ") + ")";
            var ret = "";
            if (it.sequence) ret += "sequence<";
            ret += type(it.idlType);
            if (it.array) {
                for (var i = 0, n = it.nullableArray.length; i < n; i++) {
                    var val = it.nullableArray[i];
                    if (val) ret += "?";
                    ret += "[]";
                }
            }
            if (it.sequence) ret += ">";
            if (it.nullable) ret += "?";

            return ret;
        };
        var const_value = function (it) {
            var tp = it. type;
            if (tp === "boolean") return it.value ? "true" : "false";
            else if (tp === "null") return "null";
            else if (tp === "Infinity") return (it.negative ? "-" : "") + "Infinity";
            else if (tp === "NaN") return "NaN";
            else if (tp === "number") return it.value;
            else return '"' + it.value + '"';
        };
        var argument = function (arg, pea) {
            var ret = extended_attributes(arg.extAttrs, pea);
            if (arg.optional) ret += "optional ";
            ret += type(arg.idlType);
            if (arg.variadic) ret += "...";
            ret += " " + arg.name;
            if (arg["default"]) ret += " = " + const_value(arg["default"]);
            return ret;
        };
        var args = function (its) {
            var res = ""
            ,   pea = ""
            ;
            for (var i = 0, n = its.length; i < n; i++) {
                var arg = its[i];
                if (arg.type === "ws") res += arg.value;
                else if (arg.type === "ws-pea") pea += arg.value;
                else {
                    res += argument(arg, pea);
                    if (i < n - 1) res += ",";
                    pea = "";
                }
            }
            return res;
        };
        var make_ext_at = function (it) {
            if (it["arguments"] === null) return it.name;
            context.unshift(it);
            var ret = it.name + "(" + (it["arguments"].length ? args(it["arguments"]) : "") + ")";
            context.shift(); // XXX need to add more contexts, but not more than needed for ReSpec
            return ret;
        };
        var extended_attributes = function (eats, pea) {
            if (!eats || !eats.length) return "";
            return "[" + eats.map(make_ext_at).join(", ") + "]" + pea;
        };
        
        var modifiers = "getter setter creator deleter legacycaller stringifier static".split(" ");
        var operation = function (it) {
            var ret = extended_attributes(it.extAttrs, curPea);
            curPea = "";
            if (it.stringifier && !it.idlType) return "stringifier;";
            for (var i = 0, n = modifiers.length; i < n; i++) {
                var mod = modifiers[i];
                if (it[mod]) ret += mod + " ";
            }
            ret += type(it.idlType) + " ";
            if (it.name) ret += it.name;
            ret += "(" + args(it["arguments"]) + ");";
            return ret;
        };

        var attribute = function (it) {
            var ret = extended_attributes(it.extAttrs, curPea);
            curPea = "";
            if (it["static"]) ret += "static ";
            if (it.stringifier) ret += "stringifier ";
            if (it.readonly) ret += "readonly ";
            if (it.inherit) ret += "inherit ";
            ret += "attribute " + type(it.idlType) + " " + it.name + ";";
            return ret;
        };
        
        var interface_ = function (it) {
            var ret = extended_attributes(it.extAttrs, curPea);
            curPea = "";
            if (it.partial) ret += "partial ";
            ret += "interface " + it.name + " ";
            if (it.inheritance) ret += ": " + it.inheritance + " ";
            ret += "{" + iterate(it.members) + "};";
            return ret;
        };
        
        var dictionary = function (it) {
            var ret = extended_attributes(it.extAttrs, curPea);
            curPea = "";
            if (it.partial) ret += "partial ";
            ret += "dictionary " + it.name + " ";
            ret += "{" + iterate(it.members) + "};";
            return ret;
        };
        var field = function (it) {
            var ret = extended_attributes(it.extAttrs, curPea);
            curPea = "";
            ret += type(it.idlType) + " " + it.name;
            if (it["default"]) ret += " = " + const_value(it["default"]);
            ret += ";";
            return ret;
        };
        var exception = function (it) {
            var ret = extended_attributes(it.extAttrs, curPea);
            curPea = "";
            ret += "exception " + it.name + " ";
            if (it.inheritance) ret += ": " + it.inheritance + " ";
            ret += "{" + iterate(it.members) + "};";
            return ret;
        };
        var const_ = function (it) {
            var ret = extended_attributes(it.extAttrs, curPea);
            curPea = "";
            return ret + "const " + type(it.idlType) + " " + it.name + " = " + const_value(it.value) + ";";
        };
        var typedef = function (it) {
            var ret = extended_attributes(it.extAttrs, curPea);
            curPea = "";
            ret += "typedef " + extended_attributes(it.typeExtAttrs, curTPea);
            curTPea = "";
            return ret + type(it.idlType) + " " + it.name + ";";
        };
        var implements_ = function (it) {
            var ret = extended_attributes(it.extAttrs, curPea);
            curPea = "";
            return ret + it.target + " implements " + it["implements"] + ";";
        };
        var callback = function (it) {
            var ret = extended_attributes(it.extAttrs, curPea);
            curPea = "";
            return ret + "callback " + it.name + " = " + type(it.idlType) +
                   "(" + args(it["arguments"]) + ");";
        };
        var enum_ = function (it) {
            var ret = extended_attributes(it.extAttrs, curPea);
            curPea = "";
            ret += "enum " + it.name + " {";
            for (var i = 0, n = it.values.length; i < n; i++) {
                var v = it.values[i];
                if (typeof v === "string") ret += '"' + v + '"';
                else if (v.type === "ws") ret += v.value;
                else if (v.type === ",") ret += ",";
            }
            return ret + "};";
        };
        
        var table = {
            ws:                     literal
        ,   "ws-pea":               wsPea
        ,   "ws-tpea":              wsTPea
        ,   "line-comment":         lineComment
        ,   "multiline-comment":    multilineComment
        ,   "interface":            interface_
        ,   operation:              operation
        ,   attribute:              attribute
        ,   dictionary:             dictionary
        ,   field:                  field
        ,   exception:              exception
        ,   "const":                const_
        ,   typedef:                typedef
        ,   "implements":           implements_
        ,   callback:               callback
        ,   "enum":                 enum_
        };
        var dispatch = function (it) {
            return table[it.type](it);
        };
        var iterate = function (things) {
            if (!things) return;
            var ret = "";
            for (var i = 0, n = things.length; i < n; i++) ret += dispatch(things[i]);
            return ret;
        };
        return iterate(ast);
    };


    var inNode = typeof module !== "undefined" && module.exports
    ,   obj = {
            write:  function (ast, opt) {
                if (!opt) opt = {};
                return write(ast, opt);
            }
    };

    if (inNode) module.exports = obj;
    else        window.WebIDL2Writer = obj;
    
}());
