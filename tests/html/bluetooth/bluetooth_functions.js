function clear() {
    document.getElementById("log").textContent = "";
}

function log(line) {
    document.getElementById("log").textContent += line + '\n';
}

function AsciiToDecimal(bytestr) {
    var result = [];
    for(i = 0; i < bytestr.length; i++) {
        result[i] = bytestr[i].charCodeAt(0) ;
    }
    return result;
}

function populate(testCases){
	for(i = 0; i < testCases.length; ++i) {
        var btn = document.createElement('button');
        btn.setAttribute('onclick','onButtonClick(' + i + ')');
        btn.innerHTML = 'Test '+ (i+1);
        document.getElementById('buttons').appendChild(btn);
    }
}

function timeStamp() {
    return '(' + Math.round(new Date().getTime()/1000) + ') ';
}
