function myLoad(){
	$$('[role="tree"]').each(function(elm){
		// for each node where [role="tree"], create a new Aria.Tree instance and append it to array Aria.Trees
		Aria.Trees.push(new Aria.Tree(elm));
	});
}
Event.observe(window, 'load', myLoad); // will probably use onDOMContentLoaded instead of onLoad

