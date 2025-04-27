/**
 * Icon Conversion Script for AI Chat Assistant
 * 
 * This script uses sharp to convert SVG icons to different PNG sizes
 * Required packages: sharp, fs-extra
 * 
 * Install dependencies:
 * npm install sharp fs-extra
 */

const sharp = require('sharp');
const fs = require('fs-extra');
const path = require('path');

const SOURCE_DIR = __dirname;
const DEST_DIR = path.join(__dirname, 'generated');
const SVG_FILES = {
  main: path.join(SOURCE_DIR, 'icon.svg'),
  flat: path.join(SOURCE_DIR, 'flat_icon.svg'),
  colorful: path.join(SOURCE_DIR, 'colorful_icon.svg'),
  simplified: path.join(SOURCE_DIR, 'simplified_icon.svg'),
};

// Icon sizes needed for Tauri and Windows
const SIZES = [16, 30, 32, 44, 64, 71, 89, 107, 128, 142, 150, 256, 284, 310, 512, 1024];

// Create destination directory
fs.ensureDirSync(DEST_DIR);

async function convertSvgToPng(svgPath, outputName, sizes = SIZES) {
  console.log(`Converting ${path.basename(svgPath)} to various sizes...`);
  
  for (const size of sizes) {
    const outputPath = path.join(DEST_DIR, `${outputName}-${size}x${size}.png`);
    
    try {
      await sharp(svgPath)
        .resize(size, size)
        .png()
        .toFile(outputPath);
      
      console.log(`Created: ${outputPath}`);
    } catch (error) {
      console.error(`Error converting ${svgPath} to ${size}x${size}:`, error);
    }
  }
}

async function main() {
  console.log('AI Chat Assistant Icon Converter');
  console.log('--------------------------------');
  
  // Check if SVG files exist
  for (const [name, file] of Object.entries(SVG_FILES)) {
    if (!fs.existsSync(file)) {
      console.error(`Error: SVG file not found: ${file}`);
      continue;
    }
  }
  
  // Convert main icon
  await convertSvgToPng(SVG_FILES.main, 'icon');
  
  // Convert flat icon
  if (fs.existsSync(SVG_FILES.flat)) {
    await convertSvgToPng(SVG_FILES.flat, 'flat-icon');
  }
  
  // Convert colorful icon
  if (fs.existsSync(SVG_FILES.colorful)) {
    await convertSvgToPng(SVG_FILES.colorful, 'colorful-icon');
  }
  
  // Convert simplified icon
  if (fs.existsSync(SVG_FILES.simplified)) {
    await convertSvgToPng(SVG_FILES.simplified, 'simplified-icon');
  }
  
  console.log('\nConversion complete!');
  console.log(`PNG files have been saved to: ${DEST_DIR}`);
  console.log('\nNext steps:');
  console.log('1. Use the PNG files to create .ico and .icns files');
  console.log('2. Replace the existing icons in the src-tauri/icons directory');
}

main().catch(err => {
  console.error('Conversion failed:', err);
  process.exit(1);
}); 