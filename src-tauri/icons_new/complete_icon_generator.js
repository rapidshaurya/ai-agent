/**
 * Complete Icon Generator for AI Chat Assistant
 * 
 * This script:
 * 1. Converts SVG files to PNG in various sizes
 * 2. Creates Windows-specific icon sizes
 * 3. Generates macOS iconset files
 * 4. Creates .ico file for Windows (if ImageMagick is available)
 * 
 * Required packages: sharp, fs-extra
 * Optional: child_process (for running external commands)
 */

const sharp = require('sharp');
const fs = require('fs-extra');
const path = require('path');
const { exec } = require('child_process');

// Configuration
const SOURCE_DIR = __dirname;
const GENERATED_DIR = path.join(SOURCE_DIR, 'generated');
const OUTPUT_DIR = path.join(SOURCE_DIR, '..', 'icons_updated');
const ICONSET_DIR = path.join(SOURCE_DIR, 'iconset');

// SVG source files
const SVG_FILES = {
  main: path.join(SOURCE_DIR, 'icon.svg'),
  flat: path.join(SOURCE_DIR, 'flat_icon.svg'),
  colorful: path.join(SOURCE_DIR, 'colorful_icon.svg'),
  simplified: path.join(SOURCE_DIR, 'simplified_icon.svg'),
};

// Define all required sizes
const STANDARD_SIZES = [16, 32, 64, 128, 256, 512, 1024];
const WINDOWS_SIZES = [30, 44, 71, 89, 107, 142, 150, 284, 310];
const ALL_SIZES = [...new Set([...STANDARD_SIZES, ...WINDOWS_SIZES])];

// Create output directories
fs.ensureDirSync(GENERATED_DIR);
fs.ensureDirSync(OUTPUT_DIR);

/**
 * Convert an SVG file to PNG in various sizes
 */
async function convertSvgToPng(svgPath, outputPrefix, sizes = ALL_SIZES) {
  if (!fs.existsSync(svgPath)) {
    console.error(`SVG file not found: ${svgPath}`);
    return false;
  }

  console.log(`Converting ${path.basename(svgPath)} to ${sizes.length} sizes...`);
  
  for (const size of sizes) {
    const outputPath = path.join(GENERATED_DIR, `${outputPrefix}-${size}x${size}.png`);
    
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
  
  return true;
}

/**
 * Generate macOS iconset from PNG files
 */
async function generateMacIconset(sourcePrefix) {
  if (process.platform !== 'darwin') {
    console.log('Skipping macOS iconset generation (not on macOS)');
    return false;
  }

  console.log('\nGenerating macOS iconset...');
  
  // Create iconset directory
  fs.ensureDirSync(ICONSET_DIR);
  
  // Copy files to iconset with correct naming
  const iconsetMapping = [
    { size: 16, name: 'icon_16x16.png' },
    { size: 32, name: 'icon_16x16@2x.png' },
    { size: 32, name: 'icon_32x32.png' },
    { size: 64, name: 'icon_32x32@2x.png' },
    { size: 128, name: 'icon_128x128.png' },
    { size: 256, name: 'icon_128x128@2x.png' },
    { size: 256, name: 'icon_256x256.png' },
    { size: 512, name: 'icon_256x256@2x.png' },
    { size: 512, name: 'icon_512x512.png' },
    { size: 1024, name: 'icon_512x512@2x.png' }
  ];
  
  for (const mapping of iconsetMapping) {
    const sourcePath = path.join(GENERATED_DIR, `${sourcePrefix}-${mapping.size}x${mapping.size}.png`);
    const destPath = path.join(ICONSET_DIR, mapping.name);
    
    if (fs.existsSync(sourcePath)) {
      fs.copySync(sourcePath, destPath);
      console.log(`Copied ${sourcePath} → ${destPath}`);
    } else {
      console.error(`Missing source file: ${sourcePath}`);
    }
  }
  
  // Use iconutil to create .icns file
  return new Promise((resolve) => {
    exec(`iconutil -c icns "${ICONSET_DIR}" -o "${path.join(OUTPUT_DIR, 'icon.icns')}"`, (error, stdout, stderr) => {
      if (error) {
        console.error(`Error creating .icns file: ${error.message}`);
        resolve(false);
      } else {
        console.log(`Created icon.icns in ${OUTPUT_DIR}`);
        resolve(true);
      }
    });
  });
}

/**
 * Generate Windows .ico file using ImageMagick
 */
async function generateWindowsIco(sourcePrefix) {
  console.log('\nGenerating Windows .ico file...');
  
  // Check if ImageMagick is installed
  return new Promise((resolve) => {
    exec('convert -version', async (error) => {
      if (error) {
        console.log('ImageMagick not found, skipping .ico generation');
        resolve(false);
        return;
      }
      
      // Prepare list of files for the convert command
      const sizes = [16, 32, 48, 64, 128, 256];
      const sourceFiles = sizes
        .map(size => path.join(GENERATED_DIR, `${sourcePrefix}-${size}x${size}.png`))
        .filter(file => fs.existsSync(file));
      
      if (sourceFiles.length === 0) {
        console.error('No source files found for .ico generation');
        resolve(false);
        return;
      }
      
      const outputPath = path.join(OUTPUT_DIR, 'icon.ico');
      const command = `convert ${sourceFiles.join(' ')} "${outputPath}"`;
      
      exec(command, (error, stdout, stderr) => {
        if (error) {
          console.error(`Error creating .ico file: ${error.message}`);
          resolve(false);
        } else {
          console.log(`Created icon.ico in ${OUTPUT_DIR}`);
          resolve(true);
        }
      });
    });
  });
}

/**
 * Create Windows-specific icon files with appropriate names
 */
function createWindowsIcons(sourcePrefix) {
  console.log('\nCreating Windows-specific icons...');
  
  const windowsIcons = [
    { size: 30, name: 'Square30x30Logo.png' },
    { size: 44, name: 'Square44x44Logo.png' },
    { size: 71, name: 'Square71x71Logo.png' },
    { size: 89, name: 'Square89x89Logo.png' },
    { size: 107, name: 'Square107x107Logo.png' },
    { size: 142, name: 'Square142x142Logo.png' },
    { size: 150, name: 'Square150x150Logo.png' },
    { size: 284, name: 'Square284x284Logo.png' },
    { size: 310, name: 'Square310x310Logo.png' },
    { size: 150, name: 'StoreLogo.png' }
  ];
  
  for (const icon of windowsIcons) {
    const sourcePath = path.join(GENERATED_DIR, `${sourcePrefix}-${icon.size}x${icon.size}.png`);
    const destPath = path.join(OUTPUT_DIR, icon.name);
    
    if (fs.existsSync(sourcePath)) {
      fs.copySync(sourcePath, destPath);
      console.log(`Created ${icon.name}`);
    } else {
      console.warn(`Missing source file for ${icon.name}: ${sourcePath}`);
    }
  }
}

/**
 * Create standard Tauri icon files
 */
function createStandardIcons(sourcePrefix) {
  console.log('\nCreating standard Tauri icons...');
  
  const standardIcons = [
    { size: 32, name: '32x32.png' },
    { size: 128, name: '128x128.png' },
    { size: 256, name: '128x128@2x.png' }
  ];
  
  for (const icon of standardIcons) {
    const sourcePath = path.join(GENERATED_DIR, `${sourcePrefix}-${icon.size}x${icon.size}.png`);
    const destPath = path.join(OUTPUT_DIR, icon.name);
    
    if (fs.existsSync(sourcePath)) {
      fs.copySync(sourcePath, destPath);
      console.log(`Created ${icon.name}`);
    } else {
      console.warn(`Missing source file for ${icon.name}: ${sourcePath}`);
    }
  }
  
  // Also copy the full size PNG
  const appIconPath = path.join(GENERATED_DIR, `${sourcePrefix}-1024x1024.png`);
  if (fs.existsSync(appIconPath)) {
    fs.copySync(appIconPath, path.join(OUTPUT_DIR, 'icon.png'));
    console.log('Created icon.png');
  }
}

/**
 * Clean up temporary directories
 */
function cleanup() {
  try {
    if (fs.existsSync(ICONSET_DIR)) {
      fs.removeSync(ICONSET_DIR);
      console.log('Cleaned up iconset directory');
    }
  } catch (error) {
    console.error('Error during cleanup:', error);
  }
}

/**
 * Main execution function
 */
async function main() {
  try {
    console.log('AI Chat Assistant Complete Icon Generator');
    console.log('=======================================');
    
    // Determine which SVG file to use
    let sourceFile = SVG_FILES.main;
    let sourcePrefix = 'icon';
    
    if (process.argv.length > 2) {
      const requestedStyle = process.argv[2].toLowerCase();
      
      if (requestedStyle === 'flat' && fs.existsSync(SVG_FILES.flat)) {
        sourceFile = SVG_FILES.flat;
        sourcePrefix = 'flat-icon';
      } else if (requestedStyle === 'colorful' && fs.existsSync(SVG_FILES.colorful)) {
        sourceFile = SVG_FILES.colorful;
        sourcePrefix = 'colorful-icon';
      } else if (requestedStyle === 'simplified' && fs.existsSync(SVG_FILES.simplified)) {
        sourceFile = SVG_FILES.simplified;
        sourcePrefix = 'simplified-icon';
      }
    }
    
    console.log(`Using source file: ${path.basename(sourceFile)}`);
    
    // Step 1: Convert SVG to PNGs
    const conversionSuccess = await convertSvgToPng(sourceFile, sourcePrefix);
    if (!conversionSuccess) {
      console.error('PNG conversion failed. Exiting.');
      return;
    }
    
    // Step 2: Create standard Tauri icons
    createStandardIcons(sourcePrefix);
    
    // Step 3: Create Windows-specific icons
    createWindowsIcons(sourcePrefix);
    
    // Step 4: Generate macOS iconset and .icns file
    await generateMacIconset(sourcePrefix);
    
    // Step 5: Generate Windows .ico file
    await generateWindowsIco(sourcePrefix);
    
    console.log('\n✅ Icon generation complete!');
    console.log(`All output files have been saved to: ${OUTPUT_DIR}`);
    console.log('\nTo use these icons in your application:');
    console.log(`1. Copy all files from ${OUTPUT_DIR} to src-tauri/icons/`);
    console.log('2. Rebuild your application with: npm run tauri build');
  } catch (error) {
    console.error('Error during icon generation:', error);
  } finally {
    // Clean up temporary files
    cleanup();
  }
}

// Run the main function
main().catch(console.error); 