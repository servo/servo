def main(request, response):
    headers = [(b"Content-Type", b"text/html")]
    headers += [(b"Clear-Site-Data", b'"prefetchCache"')]
    content = f'''
        <script>
            setTimeout(() => {{
                if(window.opener) {{
                    window.opener.postMessage("message", "*");
                }} else {{
                    window.parent.postMessage("message", "*");
                }}
                window.close();
            }}, 1000);
        </script>
        <body>
            {request.url}
        </body>'''
    return 200, headers, content
