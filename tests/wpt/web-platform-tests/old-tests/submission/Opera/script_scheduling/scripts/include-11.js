log("external script before adding iframe");
var iframe = document.createElement("iframe");
iframe.src = "data:text/html,<script>parent.log('script in iframe')</script>"
document.body.appendChild(iframe);