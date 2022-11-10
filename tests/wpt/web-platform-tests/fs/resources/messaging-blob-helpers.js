'use strict';

// Creates a blob URL with the contents of 'message-target.html'. Use the
// blob as an iframe src or a window.open() URL, which creates a same origin
// message target.
async function create_message_target_blob_url(test) {
  const html = await create_message_target_html_without_subresources(test);
  const blob = new Blob([html], { type: 'text/html' });
  return URL.createObjectURL(blob);
}

// Creates a data URI with the contents of 'message-target.html'. Use the
// data URI as an iframe src, which creates a cross origin message target.
async function create_message_target_data_uri(test) {
  const iframe_html =
    await create_message_target_html_without_subresources(test);
  return `data:text/html,${encodeURIComponent(iframe_html)}`;
}

// Constructs a version of 'message-target.html' without any subresources.
// Enables the creation of blob URLs, data URIs and iframe srcdocs re-using
// the contents of 'message-target.html'.
async function create_message_target_html_without_subresources(test) {
  const test_helpers_script = await fetch_text('resources/test-helpers.js');

  const messaging_helpers_script =
    await fetch_text('resources/messaging-helpers.js');

  const messaging_serialize_helpers_script =
    await fetch_text('resources/messaging-serialize-helpers.js');

  const message_target_script =
    await fetch_text('resources/message-target.js');

  // Get the inline script code from 'message-target.html'.
  const iframe = await add_iframe(test, { src: 'resources/message-target.html' });
  const iframe_script =
    iframe.contentWindow.document.getElementById('inline_script').outerHTML;
  iframe.remove();

  return '<!DOCTYPE html>' +
    `<script>${test_helpers_script}</script>` +
    `<script>${messaging_serialize_helpers_script}</script>` +
    `<script>${message_target_script}</script>` +
    `${iframe_script}`;
}

async function fetch_text(url) {
  const response = await fetch(url);
  return await response.text();
}
