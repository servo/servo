<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="utf-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <meta name="generator" content="rustdoc">
    <meta name="description" content="API documentation for the Rust `servo` crate.">
    <meta name="keywords" content="rust, rustlang, rust-lang, servo">
    <title>Supported CSS properties - servo - Rust</title>
    <link rel="stylesheet" type="text/css" href="../rustdoc.css">
    <link rel="stylesheet" type="text/css" href="../main.css">
</head>
<body class="rustdoc">
    <!--[if lte IE 8]>
    <div class="warning">
        This old browser is unsupported and will most likely display funky
        things.
    </div>
    <![endif]-->
    <section id='main' class="content mod">
      <h1 class='fqn'><span class='in-band'>CSS properties currently supported in <a class='mod' href=''>Servo</a></span></h1>
      <div id='properties' class='docblock'>
        <em>Loading</em>
      </div>
    </section>

    <script src="../jquery.js"></script>
    <script>
    (function() {
      "use strict";
      //
      // Renders list of properties into a table.
      //
      function updatePropertyList (properties) {
        var compiledTable = $('<table></table>');
        
        var header = $('<tr></tr>');
        header.append('<th>Property</th>');
        header.append('<th>Flag</td>');
        header.append('<th>Shorthand</th>');
        compiledTable.append(header);

        for (var property in properties) {
          if (properties.hasOwnProperty(property)) {
            var tr = $('<tr></tr>');
            tr.append('<td>' + property + '</td>');
            if (!properties[property].flag) {
              tr.append('<td>-</td>');
            } else {
              tr.append('<td>' + properties[property].flag + '</td>');
            }
            tr.append('<td>' + properties[property].shorthand + '</td>');
            compiledTable.append(tr);
          }
        }


        $('#properties').html(compiledTable);
      }

      $.get('./css-properties.json').success(updatePropertyList).error(function () {
        $('#properties').html("<p>Unable to load json.</p>");
      });
    }());
    </script>
</body>
</html>
