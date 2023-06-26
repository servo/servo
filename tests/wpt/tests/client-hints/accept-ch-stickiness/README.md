These tests all follow the same format, calling the `run_test` function from
`resources/accept_ch_test.js`. This function does the following:

 * checks to make sure no client-hint preferences are saved for a particular origin
 * loading a page with the response header `Accept-CH: device-memory` via a
   particular method:
    * Navigation (via window.open)
    * Subresource (via fetch)
    * iframe (added via js)
 * Navigates to another page to check if the device-memory client hint was sent
   with the next request

Each test is in a separate file to ensure that the browser and it's state is
properly reset between each test.
