$body = '{}'
$response_load = Invoke-RestMethod -Method Post `
                              -Uri "http://127.0.0.1:7000/session/${SESSIONID}/element/${elementID}/click" `
                              -Body $body `
                              -ContentType "application/json" `
                              -Verbose
