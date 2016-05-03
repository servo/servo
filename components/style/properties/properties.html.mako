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
        <table>
          <tr>
            <th>Property</th>
            <th>Flag</th>
            <th>Shorthand</th>
          </tr>
          % for prop in properties:
            <tr>
              <td>${prop}</td>
              <td>${properties[prop]['flag']}</td>
              <td>${properties[prop]['shorthand']}</td>
            </tr>
          % endfor
        </table>
      </div>
    </section>
</body>
</html>
