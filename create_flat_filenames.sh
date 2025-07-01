#!/bin/bash
# File Extraction Script for Distli Mesh BC
# Creates copies of all source files with full directory path as filename pattern
echo "🚀 Distli Mesh BC - File Extraction Script"
echo "=========================================="

# Create output directory for extracted files
OUTPUT_DIR="extracted_files"
mkdir -p "$OUTPUT_DIR"
echo "📁 Creating extracted files in: $OUTPUT_DIR/"
echo ""

# Function to recursively extract files from a directory
extract_files_recursive() {
    local source_dir="$1"
    
    if [ ! -d "$source_dir" ]; then
        echo "⚠️  Directory $source_dir not found, skipping..."
        return
    fi
    
    echo "📂 Processing $source_dir/ ..."
    
    # Find all files recursively in the source directory
    find "$source_dir" -type f | while read -r file; do
        # Get the relative path from current directory
        relative_path="$file"
        
        # Convert path separators to colons and remove leading ./
        new_filename=$(echo "$relative_path" | sed 's|^\./||' | sed 's|/|:|g')
        
        # Copy file to output directory
        if cp "$file" "$OUTPUT_DIR/$new_filename"; then
            echo "  ✅ $file → $new_filename"
        else
            echo "  ❌ Failed to copy $file"
        fi
    done
}

# Function to extract files from specific directories (non-recursive)
extract_files_single() {
    local source_dir="$1"
    
    if [ ! -d "$source_dir" ]; then
        echo "⚠️  Directory $source_dir not found, skipping..."
        return
    fi
    
    echo "📂 Processing $source_dir/ (single level)..."
    
    # Find files only in the immediate directory (not subdirectories)
    find "$source_dir" -maxdepth 1 -type f | while read -r file; do
        # Get the relative path from current directory
        relative_path="$file"
        
        # Convert path separators to colons and remove leading ./
        new_filename=$(echo "$relative_path" | sed 's|^\./||' | sed 's|/|:|g')
        
        # Copy file to output directory
        if cp "$file" "$OUTPUT_DIR/$new_filename"; then
            echo "  ✅ $file → $new_filename"
        else
            echo "  ❌ Failed to copy $file"
        fi
    done
}

echo "🔍 Extracting files from all directories..."
echo ""

# Extract files recursively from main source directories
extract_files_recursive "src"

# Extract files from other directories (single level)
extract_files_single "public"
extract_files_single "docker"

# Copy root configuration files
echo "📂 Processing root configuration files..."
for file in Cargo.toml README.md LICENSE CHANGELOG.md .gitignore package.json tsconfig.json; do
    if [ -f "$file" ]; then
        # For root files, just prefix with "root-"
        new_filename="root-$file"
        if cp "$file" "$OUTPUT_DIR/$file"; then
            echo "  ✅ $file → $file"
        else
            echo "  ❌ Failed to copy $file"
        fi
    fi
done

echo ""
echo "📊 Extraction Summary:"
echo "====================="

# Count and list all extracted files
total_files=$(find "$OUTPUT_DIR" -type f | wc -l)
echo "📁 Total files extracted: $total_files"
echo ""

echo "📋 Extracted files:"
ls -la "$OUTPUT_DIR/" | grep -v '^d' | awk '{print "  📄 " $9}' | sort
echo ""

echo "🎉 File extraction complete!"
echo "📁 All files available in: $OUTPUT_DIR/"
echo ""
echo "💡 Usage: These files can be easily uploaded, shared, or reviewed individually"
echo "   Each file is named with its full directory path for easy identification"
echo ""
echo "🔍 Example naming:"
echo "   src/common/mod.rs → src:common:mod.rs"
echo "   src/tracker/utils.rs → src:tracker:utils.rs"
echo "   src/enterprise_bc/config/settings.rs → src:enterprise_bc:config:settings.rs"
