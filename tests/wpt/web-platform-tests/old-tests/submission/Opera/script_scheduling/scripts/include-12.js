log("external script before adding object");
var object = document.createElement("object");
object.data = "data:text/html,<script>parent.log('script in object')</script>"
document.body.appendChild(object);