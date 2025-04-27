# Icon Update Guide for AI Chat Assistant

This guide explains how to update the application icon for the AI Chat Assistant app.

## Overview

The application uses:
- SVG source files (in the `icons_new` directory)
- A conversion script that generates PNG files in various sizes
- Tauri configuration that references specific icon files

## Step-by-Step Instructions

### 1. Update or Replace SVG Source Files

Edit or replace one of these SVG files in the `src-tauri/icons_new` directory:
- `simplified_icon.svg` - The main icon with a blue gradient and chat bubble
- `colorful_icon.svg` - A more colorful version of the icon
- `flat_icon.svg` - A flat version of the icon

You can use any SVG editor like Inkscape, Adobe Illustrator, or Figma.

### 2. Run the Conversion Script

The `convert_icons.js` script converts your SVG files to the required PNG formats:

```bash
cd ai-app/src-tauri/icons_new
npm install sharp fs-extra  # Only needed first time
node convert_icons.js
```

This will create PNG files of various sizes in a `generated` subdirectory.

### 3. Replace the Icon Files

Copy the generated PNG files to replace the existing icon files in `src-tauri/icons`:
- Copy all PNGs with appropriate sizes
- For `.ico` and `.icns` files, you'll need to use a converter:
  - For macOS: `iconutil` for `.icns` 
  - For Windows: Use a tool like `imagemagick` or an online converter for `.ico`

### 4. Update Tauri Configuration (if needed)

If you've changed icon filenames, update the `tauri.conf.json` file in the `bundle.icon` section:

```json
"icon": [
  "icons/32x32.png",
  "icons/128x128.png",
  "icons/128x128@2x.png",
  "icons/icon.icns",
  "icons/icon.ico"
]
```

### 5. Rebuild the Application

Rebuild your application to apply the icon changes:

```bash
cd ai-app
npm run tauri build
```

## Tips

- Maintain the original dimensions and aspect ratio of the SVG files
- The icon uses a blue gradient background with a modern chat bubble design
- Test the icons at small sizes to ensure they remain clear and recognizable
- Different platforms (Windows, macOS) have different icon requirements

## Troubleshooting

- If icons don't update, clear the build cache and rebuild
- Ensure all required icon sizes are generated
- Check that the paths in `tauri.conf.json` match your actual icon file locations 