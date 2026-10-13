# Payment Method Manifest Web Platform Tests

This directory contains Web Platform Tests (WPTs) for the [Payment Method
Manifest specification](https://w3c.github.io/payment-method-manifest/).

## Testing methdology

In order to minimize reliance upon other specifications such as the [Web-based
Payment Handler API](https://w3c.github.io/web-based-payment-handler/), these
tests try to avoid expecting any particular behavior around the payment handler
itself. Instead, tests monitor for events that the browser should send to the
server(s) based on the Payment Method Manifest specification and verify only
that the correct events are sent or not sent.
