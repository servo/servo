def main(request, response):
    headers = [(b"Content-Type", b"text/html")]
    return headers, u'''
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
