<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="utf-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Supported CSS properties in Servo</title>
    <link rel="stylesheet" type="text/css" href="../normalize.css">
    <link rel="stylesheet" type="text/css" href="../rustdoc.css">
    <link rel="stylesheet" type="text/css" href="../light.css">
</head>
<body class="rustdoc">
    <section id='main' class="content mod">
      <h1 class='fqn'><span class='in-band'>CSS properties currently supported in Servo</span></h1>
      % for kind, props in sorted(properties.items()):
      <h2>${kind.capitalize()}</h2>
      <table>
        <tr>
          <th>Name</th>
          <th>Pref</th>
        </tr>
        % for name, data in sorted(props.items()):
          <tr>
            <td><code>${name}</code></td>
            <td><code>${data['pref'] or ''}</code></td>
          </tr>
        % endfor
      </table>
      % endfor
    </section>
</body>
</html>
