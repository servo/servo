
var xhr=new XMLHttpRequest()
xhr.onreadystatechange = function(){
	if(xhr.readyState == 4){
		var status = xhr.responseText === 'bottom\n' ? 'PASSED' : 'FAILED'
		self.postMessage(status)
	}
}
xhr.open('GET', 'folder.txt', true)
xhr.send()
