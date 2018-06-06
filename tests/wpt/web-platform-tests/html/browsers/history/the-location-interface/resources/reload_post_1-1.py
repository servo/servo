def main(request, response):
    headers = [("Content-Type", "text/html")]
    return headers, '''
        <script>
            onload = function() {opener.next()}
            document.write(Math.random());
        </script>
        <form method="POST" action="">
            <input type=hidden name=test value=test>
            <input type=submit>
        </form>
        <button onclick="location.reload()">Reload</button>
    '''
