function createImageWithMetadata({
    width,
    height,
    preferredWidth,
    preferredHeight,
    resolutionX,
    resolutionY,
    resolutionUnit,
    orientation
}) {
    const canvas = document.createElement('canvas')
    canvas.width = width || 100
    canvas.height = height || 100
    const ctx = canvas.getContext('2d')
    ctx.fillColor = 'green'
    ctx.fillRect(0, 0, canvas.width, canvas.height)
    const original = canvas.toDataURL('image/jpeg')
    const root = {}
    const exif = {}
    if (orientation !== undefined)
        root[piexif.ExifIFD.Orientation] = orientation
    if (resolutionX !== undefined)
        root[piexif.ImageIFD.XResolution] = Array.isArray(resolutionX) ? resolutionX : [resolutionX, 1]
    if (resolutionY !== undefined)
        root[piexif.ImageIFD.YResolution] = Array.isArray(resolutionY) ? resolutionY : [resolutionY, 1]
    if (resolutionUnit !== undefined)
        root[piexif.ImageIFD.ResolutionUnit] = resolutionUnit
    if (preferredWidth !== undefined)
        exif[piexif.ExifIFD.PixelXDimension] = preferredWidth
    if (preferredHeight !== undefined)
        exif[piexif.ExifIFD.PixelYDimension] = preferredHeight
    const exifString = piexif.dump({'0th': root, 'Exif': exif})
    const newDataUrl = piexif.insert(exifString, original)
    const image = new Image()
    image.src = newDataUrl
    return new Promise(resolve => {
        image.onload = () => resolve(image);
    })
}