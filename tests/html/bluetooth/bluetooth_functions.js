function clear() {
    document.getElementById("log").textContent = "";
}

function log(line) {
    document.getElementById("log").textContent += timeStamp() + line + '\n';
}

function asciiToDecimal(bytestr) {
    var result = [];
    for(i = 0; i < bytestr.length; i++) {
        result[i] = bytestr.charCodeAt(i) ;
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
    var date = new Date;
    var hours = date.getHours();
    var minutes = "0" + date.getMinutes();
    var seconds = "0" + date.getSeconds();
    return hours + ':' + minutes.substr(-2) + ':' + seconds.substr(-2) + ' ';
}
