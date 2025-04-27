# AI Chat Assistant Icon Update Instructions

## About the Icon
A custom icon has been designed for the AI Chat Assistant application. It features a modern chat bubble with three dots on a blue gradient background, representing an AI assistant in conversation.

## Update Steps

### 1. Generate the Icon
You can use the provided SVG files and HTML template to generate PNG icons:

1. Open the `icon_template.html` file in a web browser
2. Choose which icon style you prefer
3. Take a screenshot of the icon
4. Use an image editor to crop the icon and save it as PNG

### 2. Replace Icon Files
Replace the following icon files in the `ai-app/src-tauri/icons` directory:

**Main Icon Files:**
- `icon.png` - 1024×1024px main icon
- `icon.ico` - Windows icon
- `icon.icns` - macOS icon

**Windows Icons:**
- `32x32.png` - 32×32px
- `128x128.png` - 128×128px 
- `128x128@2x.png` - 256×256px
- Various Square*Logo.png files

### 3. Converting SVG to Icon Formats
For best results, you may need to convert the SVG to the proper formats:

**For macOS (.icns):**
1. Generate PNGs in sizes: 16x16, 32x32, 64x64, 128x128, 256x256, 512x512, and 1024x1024
2. Use a tool like `iconutil` or an online converter

**For Windows (.ico):**
1. Generate PNGs in sizes: 16x16, 32x32, 48x48, 64x64, 128x128, 256x256
2. Use an online ICO converter

### 4. Updating Tauri Config
The application name has already been updated in the `tauri.conf.json` file:
- Product name: "AI Chat Assistant"
- Window title: "AI Chat Assistant"

### 5. Build the Application
After replacing the icons, rebuild the application:
```
cd ai-app
npm run tauri build
```

## Icon Design Details
The icon was designed with these considerations:
- Blue gradient color scheme to match the application theme
- Chat bubble with three dots to represent AI conversation
- Clean, modern design that works at various sizes
- Available in both circular and squared versions

The SVG source files are provided so they can be modified if needed. 