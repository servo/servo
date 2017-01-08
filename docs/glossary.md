# How to use this glossary

This is a collection of common terms that have a specific meaning in the context of the Servo project. The goal is to provide high-level definitions and useful links for further reading, rather than complete documentation about particular parts of the code.

If there is a word or phrase used in Servo's code, issue tracker, mailing list, etc. that is confusing, please make a pull request that adds it to this file with a body of `TODO`. This will signal more knowledgeable people to add a more meaningful definition.

# Glossary

### Compositor ###

The thread that receives input events from the operating system and forwards them to the constellation. It is also in charge of compositing complete renders of web content and displaying them on the screen as fast as possible.

### Constellation ###

The thread that controls a collection of related web content. This could be thought of as an owner of a single tab in a tabbed web browser; it encapsulates session history, knows about all frames in a frame tree, and is the owner of the pipeline for each contained frame.

### Display list ###

A list of concrete rendering instructions. The display list is post-layout, so all items have stacking-context-relative pixel positions, and z-index has already been applied, so items later in the display list will always be on top of items earlier in it.

### Layout thread ###

A thread that is responsible for laying out a DOM tree into layers of boxes for a particular document. Receives commands from the script thread to lay out a page and either generate a new display list for use by the renderer thread, or return the results of querying the layout of the page for use by script.

### Pipeline ###

A unit encapsulating a means of communication with the script, layout, and renderer threads for a particular document. Each pipeline has a globally-unique id which can be used to access it from the constellation.

### Renderer thread (alt. paint thread) ###

A thread which translates a display list into a series of drawing commands that render the contents of the associated document into a buffer, which is then sent to the compositor.

### Script thread (alt. script task) ###

A thread that executes JavaScript and stores the DOM representation of all documents that share a common [origin](https://tools.ietf.org/html/rfc6454). This thread translates input events received from the constellation into DOM events [per the specification](https://w3c.github.io/uievents/), invokes the HTML parser when new page content is received, and evaluates JS for events like timers and `<script>` elements.
