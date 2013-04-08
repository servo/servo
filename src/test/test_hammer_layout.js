window.setTimeout(function() {
  //var divs = document.getElementsByTagName("div");
//  divs[0].setAttribute('id', 'styled');
  function print_tree(n) {
    window.alert(n.nodeType);
    //window.alert(n.tagName);
    n = n.firstChild;
    while (n) {
      print_tree(n);
      n = n.nextSibling;
    }
  }
  print_tree(document.documentElement);
//window.alert(document.documentElement.tagName);
//window.alert(document.documentElement.firstChild.nodeType);
//window.alert(document.documentElement.firstChild.firstChild.nodeType);
}, 200);