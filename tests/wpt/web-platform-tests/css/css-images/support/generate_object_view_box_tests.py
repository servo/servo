import sys


def generate_file(source, destination, tag, name, image_source):
    file = open(source)
    lines = file.read()
    file.close()

    replaced_lines = lines.replace('__TAG__',
                                   tag).replace('__NAME__', name).replace(
                                       '__IMAGE_SOURCE__', image_source)
    replaced_lines = '<!-- This is an autogen file. Run support/generate_object_view_box_tests.py to update -->\n' + replaced_lines
    new_file = open(destination, "w")
    new_file.write(replaced_lines)
    new_file.close()


def generate_for_object_fit(object_fit):
    names = ['img', 'svg', 'canvas', 'video']
    tags = ['img', 'img', 'canvas', 'video']
    image_sources = [
        'support/exif-orientation-6-ru.jpg',
        'support/blue-green-red-yellow-50x100.svg', '', ''
    ]

    for i in range(len(names)):
        source = 'object-view-box-fit-' + object_fit + '-template.html'
        destination = '../object-view-box-fit-' + object_fit + '-' + names[
            i] + '.html'
        generate_file(source, destination, tags[i], names[i], image_sources[i])

        source = 'object-view-box-fit-' + object_fit + '-ref-template.html'
        destination = '../object-view-box-fit-' + object_fit + '-' + names[
            i] + '-ref.html'
        generate_file(source, destination, tags[i], names[i], image_sources[i])


def generate_for_writing_mode():
    names = ['img', 'svg', 'canvas', 'video']
    tags = ['img', 'img', 'canvas', 'video']
    image_sources = [
        'support/exif-orientation-6-ru.jpg',
        'support/blue-green-red-yellow-50x100.svg', '', ''
    ]

    for i in range(len(names)):
        source = 'object-view-box-writing-mode-template.html'
        destination = '../object-view-box-writing-mode-' + names[i] + '.html'
        generate_file(source, destination, tags[i], names[i], image_sources[i])

        source = 'object-view-box-writing-mode-ref-template.html'
        destination = '../object-view-box-writing-mode-' + names[
            i] + '-ref.html'
        generate_file(source, destination, tags[i], names[i], image_sources[i])


def main():
    object_fit_types = ['fill', 'cover', 'contain', 'none']
    for object_fit in object_fit_types:
        generate_for_object_fit(object_fit)

    generate_for_writing_mode()


if __name__ == '__main__':
    sys.exit(main())
