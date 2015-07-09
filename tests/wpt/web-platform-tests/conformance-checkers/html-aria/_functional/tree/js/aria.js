
/* Aria Singleton */
var Aria = {
	Trees: new Array(), // instances of Aria.Tree Class
	isEnabled: function(inNode){
		// todo: this may need to check isEnabled on all parentNodes, inheritence of aria-enabled is ambiguous
		if(inNode.getAttribute('aria-enabled') && inNode.getAttribute('aria-enabled').toLowerCase()=='false') return false;
		else return true;
	},
	isExpanded: function(inNode){
		if(inNode.getAttribute('aria-expanded') && inNode.getAttribute('aria-expanded').toLowerCase()=='false') return false;
		else return true;
	},
	isTreeItem: function(inNode){
		if(inNode.getAttribute('role') && inNode.getAttribute('role').toLowerCase()=='treeitem') return true;
		else return false;
	}
};

Aria.Tree = Class.create();
Aria.Tree.prototype = {
	initialize: function(inNode){
		this.el = $(inNode);
		this.index = Aria.Trees.length; // each tree should know its index in the Aria singleton's list, in order to concatenate id strings
		this.strActiveDescendant = this.el.getAttribute('aria-activedescendant');
		this.strDefaultActiveDescendant = 'tree'+this.index+'_item0'; // default first item
		if(!$(this.strActiveDescendant)) this.strActiveDescendant = this.strDefaultActiveDescendant; // set to default if no existing activedescendant
		this.setActiveDescendant($(this.strActiveDescendant));
		
		// set up event delegation on the tree node
		Event.observe(this.el, 'click', this.handleClick.bindAsEventListener(this));
		Event.observe(this.el, 'keydown', this.handleKeyPress.bindAsEventListener(this)); //webkit doesn't send keypress events for arrow keys, so use keydown instead
		
	},
	getActiveDescendant: function(inNode){
		if(inNode){ // if inNode (from event target), sets the activedescendant to nearest ancestor treeitem
			var el = $(inNode);
			while(el != this.el){
				if(Aria.isTreeItem(el)) break; // exit the loop; we have the treeitem
				el = el.parentNode;
			}
			if(el == this.el) {
				this.setActiveDescendant(); // set to default activedescendant
			} else {
				this.setActiveDescendant(el);
				return el;
			}
		} else {
			return $(this.el.getAttribute('aria-activedescendant'));
		}
	},
	getNextTreeItem: function(inNode){
		var el = $(inNode);
		var originalElm = $(inNode);
		while(!Aria.isTreeItem(el) || el == originalElm){
			if(Aria.isExpanded(el) && el.down()){ // should be el.down('[role="treeitem"]');
				var elements = el.getElementsByTagName('*');
				for(var i=0, c=elements.length; i<c; i++){
					if(Aria.isTreeItem(elements[i])) return elements[i];
				}
			}
			if(el.next()){
				el = el.next();
			} else {
				while(!el.parentNode.next() && el.parentNode != this.el){
					el = el.parentNode;
				}
				if(el.parentNode == this.el) return originalElm; // if no next items in tree, return current treeitem
				else el = el.parentNode.next();
			}
		}
		return el;
	},
	getPreviousTreeItem: function(inNode){
		var el = $(inNode);
		var originalElm = $(inNode);
		while(!Aria.isTreeItem(el) || el == originalElm){
			if(el.previous()){
				el = el.previous();
				// recursively choose last child node of previous el, as long as it's not in an collapsed node
				if (el.down() && Aria.isExpanded(el)){
					el = el.down();
					while (el.next() || (el.down() && Aria.isExpanded(el))){
						if (el.next()) el = el.next();
						else el = el.down();
					}
				}
			} else {
				if(el.parentNode == this.el) return originalElm; // if no previous items in tree, return current treeitem
				el = el.parentNode;
			}
		}
		if(el == this.el) return originalElm; // if no previous items in tree, return current treeitem
		return el;
	},
	handleClick: function(inEvent){
		var target = inEvent.target; // get the click target
		var el = this.getActiveDescendant(target);
		if(target.className.indexOf('expander')>-1){ // if it's an expander widget
			this.toggleExpanded(el); // toggle the aria-expanded attribute on activedescendant
			Event.stop(inEvent); // and stop the event
		}
	},
	handleKeyPress: function(inEvent){
		switch(inEvent.keyCode){
			// case Event.KEY_PAGEUP:   break;
			// case Event.KEY_PAGEDOWN: break;
			// case Event.KEY_END:      break;
			// case Event.KEY_HOME:     break;
			case Event.KEY_LEFT:     this.keyLeft();  break;
			case Event.KEY_UP:       this.keyUp();    break;
			case Event.KEY_RIGHT:    this.keyRight(); break;
			case Event.KEY_DOWN:     this.keyDown();  break;
			default:
				return;
		}
		Event.stop(inEvent);
	},
	keyLeft: function(){
		var el = this.activeDescendant;
		if(Aria.isExpanded(el)){
			el.setAttribute('aria-expanded','false');
			this.setActiveDescendant(this.activeDescendant);
		}
	},
	keyUp: function(){
		var el = this.activeDescendant;
		this.setActiveDescendant(this.getPreviousTreeItem(el));
	},
	keyRight: function(){
		var el = this.activeDescendant;
		if(!Aria.isExpanded(el)){
			el.setAttribute('aria-expanded','true');
			this.setActiveDescendant(this.activeDescendant);
		}
	},
	keyDown: function(){
		var el = this.activeDescendant;
		this.setActiveDescendant(this.getNextTreeItem(el));
	},
	setActiveDescendant: function(inNode){
		Element.removeClassName(this.activeDescendant,'activedescendant')
		if($(inNode)) this.activeDescendant = $(inNode);
		else this.activeDescendant = $(this.strDefaultActiveDescendant);
		Element.addClassName(this.activeDescendant,'activedescendant')
		this.strActiveDescendant = this.activeDescendant.id;
		this.el.setAttribute('aria-activedescendant', this.activeDescendant.id);
	},
	toggleExpanded: function(inNode){
		var el = $(inNode);
		if(Aria.isExpanded(el)){
			el.setAttribute('aria-expanded','false');
		} else {
			el.setAttribute('aria-expanded','true');	
		}
		this.setActiveDescendant(el);
	}
};
