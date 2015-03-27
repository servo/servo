// This simply posts a message to the owner page with the contents of the Referer header
var xhr=new XMLHttpRequest()
xhr.onreadystatechange = function(){
        if(xhr.readyState == 4){
                var obj = {test:'Referer header', result:xhr.responseText}
                self.postMessage(obj)
        }
}
xhr.open('GET', 'inspect-headers.py?filter_name=referer', true)
xhr.send()

// This simply posts a message to the owner page with the contents of the Origin header
var xhr2=new XMLHttpRequest()
xhr2.onreadystatechange = function(){
        if(xhr2.readyState == 4){
                var obj = {test:'Origin header', result:xhr2.responseText}
                self.postMessage(obj)
        }
}
xhr2.open('GET', location.protocol + '//www2.'+location.hostname+((location.port === "")?"":":"+location.port)+(location.pathname.replace(/[^/]*$/, ''))+'inspect-headers.py?filter_name=origin&cors', true)
xhr2.send()

// If "origin" / base URL is the origin of this JS file, we can load files
// from the server it originates from.. and requri.py will be able to tell us
// what the requested URL was
var xhr3=new XMLHttpRequest()
xhr3.onreadystatechange = function(){
        if(xhr3.readyState == 4){
                var obj = {test:'Request URL test', result:xhr3.responseText}
                self.postMessage(obj)
        }
}
xhr3.open('GET', 'requri.py?full', true)
xhr3.send()
