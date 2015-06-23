function URLTestParser(input) {
  var specialSchemes = ["ftp", "file", "gopher", "http", "https", "ws", "wss"],
      tokenMap = { "\\": "\\", "#": "#", n: "\n", r: "\r", s: " ", t: "\t", f: "\f" }
      resultMap = { s: "scheme", u: "username", pass: "password", h: "host", port: "port", p: "path", q: "query", f: "fragment" },
      results = []
  function Test() {
    this.input = ""
    this.base = ""
    this.scheme = ""
    this.username = ""
    this.password = null
    this.host = null
    this.port = ""
    this.path = ""
    this.query = ""
    this.fragment = ""
    Object.defineProperties(this, {
      "href": { get: function() {
        return !this.scheme ? this.input : this.protocol + (
          this.host != null ? "//" + (
            ("" != this.username || null != this.password) ? this.username + (
              null != this.password ? ":" + this.password : ""
            ) + "@" : ""
          ) + this.host + (this.port ? ":" + this.port : "") : ""
        ) + this.path + this.query + this.fragment
      } },
      "protocol": { get: function() { return this.scheme + ":" } },
      "search": { get: function() { return "?" == this.query ? "" : this.query } },
      "hash": { get: function() { return "#" == this.fragment ? "" : this.fragment } },
      "hostname": { get: function() { return null == this.host ? "" : this.host } }
    })
  }
  function normalize(input) {
    var output = ""
    for(var i = 0, l = input.length; i < l; i++) {
      var c = input[i]
      if(c == "\\") {
        var nextC = input[++i]
        if(tokenMap.hasOwnProperty(nextC)) {
          output += tokenMap[nextC]
        } else if(nextC == "u") {
          output += String.fromCharCode(parseInt(input[++i] + input[++i] + input[++i] + input[++i], 16))
        } else {
          throw new Error("Input is invalid.")
        }
      } else {
        output += c
      }
    }
    return output
  }
  var lines = input.split("\n")
  for(var i = 0, l = lines.length; i < l; i++) {
    var line = lines[i]
    if(line === "" || line.indexOf("#", 0) === 0) {
      continue
    }
    var pieces = line.split(" "),
        result = new Test()
    result.input = normalize(pieces.shift())
    var base = pieces.shift()
    if(base === "" || base === undefined) {
      result.base = results[results.length - 1].base
    } else {
      result.base = normalize(base)
    }
    for(var ii = 0, ll = pieces.length; ii < ll; ii++) {
      var piece = pieces[ii]
      if(piece.indexOf("#", 0) === 0) {
        continue
      }
      var subpieces = piece.split(":"),
          token = subpieces.shift()
          value = subpieces.join(":")
      result[resultMap[token]] = normalize(value)
    }
    results.push(result)
  }
  return results
}
