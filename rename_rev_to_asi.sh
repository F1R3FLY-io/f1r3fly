#!/bin/bash

# Script to replace REV -> ASI throughout the entire project
# This migrates an EXISTING blockchain from REV tokens to ASI tokens
set -e

echo "🚀 Starting REV → ASI migration for existing blockchain..."
echo ""

# 0. CLEANUP - Remove any leftover ASI files from previous runs
echo "🧹 Cleaning up leftover files from previous runs..."

# Remove ASI directories if they exist (from previous incomplete runs)
if [ -d "node/src/main/scala/coop/rchain/node/asivaultexport" ]; then
    rm -rf "node/src/main/scala/coop/rchain/node/asivaultexport"
    echo "✅ Removed leftover node/src/main/.../asivaultexport/"
fi

if [ -d "node/src/test/scala/coop/rchain/node/asivaultexport" ]; then
    rm -rf "node/src/test/scala/coop/rchain/node/asivaultexport"
    echo "✅ Removed leftover node/src/test/.../asivaultexport/"
fi

# Remove ASI files if they exist (from previous incomplete runs)
if [ -f "casper/src/main/resources/ASIVault.rho" ]; then
    rm "casper/src/main/resources/ASIVault.rho"
    echo "✅ Removed leftover ASIVault.rho"
fi

if [ -f "casper/src/main/resources/MultiSigASIVault.rho" ]; then
    rm "casper/src/main/resources/MultiSigASIVault.rho"
    echo "✅ Removed leftover MultiSigASIVault.rho"
fi

if [ -f "casper/src/test/resources/ASIVaultTest.rho" ]; then
    rm "casper/src/test/resources/ASIVaultTest.rho"
    echo "✅ Removed leftover ASIVaultTest.rho"
fi

if [ -f "casper/src/test/resources/MultiSigASIVaultTest.rho" ]; then
    rm "casper/src/test/resources/MultiSigASIVaultTest.rho"
    echo "✅ Removed leftover MultiSigASIVaultTest.rho"
fi

if [ -f "casper/src/test/resources/ASIAddressTest.rho" ]; then
    rm "casper/src/test/resources/ASIAddressTest.rho"
    echo "✅ Removed leftover ASIAddressTest.rho"
fi

if [ -f "casper/src/main/scala/coop/rchain/casper/genesis/contracts/ASIGenerator.scala" ]; then
    rm "casper/src/main/scala/coop/rchain/casper/genesis/contracts/ASIGenerator.scala"
    echo "✅ Removed leftover ASIGenerator.scala"
fi

if [ -f "rholang/src/main/scala/coop/rchain/rholang/interpreter/util/ASIAddress.scala" ]; then
    rm "rholang/src/main/scala/coop/rchain/rholang/interpreter/util/ASIAddress.scala"
    echo "✅ Removed leftover ASIAddress.scala"
fi

if [ -f "rholang/src/test/scala/coop/rchain/rholang/interpreter/util/ASIAddressSpec.scala" ]; then
    rm "rholang/src/test/scala/coop/rchain/rholang/interpreter/util/ASIAddressSpec.scala"
    echo "✅ Removed leftover ASIAddressSpec.scala"
fi

if [ -f "casper/src/test/scala/coop/rchain/casper/genesis/contracts/ASIAddressSpec.scala" ]; then
    rm "casper/src/test/scala/coop/rchain/casper/genesis/contracts/ASIAddressSpec.scala"
    echo "✅ Removed leftover ASIAddressSpec.scala"
fi

echo "🧹 Cleanup completed!"
echo ""

# 0.5. VERIFY REQUIRED FILES EXIST
echo "🔍 Verifying required REV files exist..."

required_files=(
    "casper/src/main/resources/RevVault.rho"
    "casper/src/main/resources/MultiSigRevVault.rho"
    "casper/src/test/resources/RevVaultTest.rho"
    "casper/src/test/resources/MultiSigRevVaultTest.rho"
    "casper/src/test/resources/RevAddressTest.rho"
    "casper/src/main/scala/coop/rchain/casper/genesis/contracts/RevGenerator.scala"
    "rholang/src/main/scala/coop/rchain/rholang/interpreter/util/RevAddress.scala"
    "rholang/src/test/scala/coop/rchain/rholang/interpreter/util/RevAddressSpec.scala"
    "casper/src/test/scala/coop/rchain/casper/genesis/contracts/RevAddressSpec.scala"
)

required_dirs=(
    "node/src/main/scala/coop/rchain/node/revvaultexport"
    "node/src/test/scala/coop/rchain/node/revvaultexport"
)

missing_files=()
missing_dirs=()

for file in "${required_files[@]}"; do
    if [ ! -f "$file" ]; then
        missing_files+=("$file")
    fi
done

for dir in "${required_dirs[@]}"; do
    if [ ! -d "$dir" ]; then
        missing_dirs+=("$dir")
    fi
done

if [ ${#missing_files[@]} -gt 0 ] || [ ${#missing_dirs[@]} -gt 0 ]; then
    echo "❌ ERROR: Missing required files/directories:"
    for file in "${missing_files[@]}"; do
        echo "   - $file"
    done
    for dir in "${missing_dirs[@]}"; do
        echo "   - $dir"
    done
    echo ""
    echo "💡 Please make sure you have a clean REV codebase before running this script."
    echo "   Consider doing: git checkout . && git clean -fd"
    exit 1
fi

echo "✅ All required REV files found!"
echo ""

# 1. REPLACE CONTENT IN FILES
echo "📝 Replacing text in files..."

# Main identifiers (case-sensitive) - ONLY in code files
# NOTE: Documentation files (.md, .txt, .json, .py, etc.) are NOT processed
# ASI team should update documentation themselves based on their future features
find . -type f \( -name "*.scala" -o -name "*.rs" -o -name "*.rho" \) ! -path "./.git/*" | while read -r file; do

    # Case-sensitive replacements
    sed -i '' \
        -e 's/RevVault/ASIVault/g' \
        -e 's/RevAddress/ASIAddress/g' \
        -e 's/RevGenerator/ASIGenerator/g' \
        -e 's/revVault/asiVault/g' \
        -e 's/revAddress/asiAddress/g' \
        -e 's/revGenerator/asiGenerator/g' \
        -e 's/RevAccount/ASIAccount/g' \
        -e 's/revAccount/asiAccount/g' \
        -e 's/RevAddr/ASIAddr/g' \
        -e 's/revAddr/asiAddr/g' \
        -e 's/revVaultCh/asiVaultCh/g' \
        -e 's/multiSigRevVault/multiSigASIVault/g' \
        -e 's/MultiSigRevVault/MultiSigASIVault/g' \
        -e 's/REV_ADDRESS/ASI_ADDRESS/g' \
        -e 's/rev_address/asi_address/g' \
        -e 's/revAddress/asiAddress/g' \
        -e 's/receiveRev/receiveASI/g' \
        -e 's/sendRev/sendASI/g' \
        "$file"
done

# 2. REPLACE URI IN REGISTRY
echo "🔗 Replacing Registry URIs..."
find . -type f \( -name "*.scala" -o -name "*.rs" -o -name "*.rho" \) ! -path "./.git/*" | while read -r file; do
    sed -i '' \
        -e 's/rho:rchain:revVault/rho:rchain:asiVault/g' \
        -e 's/rho:rchain:multiSigRevVault/rho:rchain:multiSigASIVault/g' \
        -e 's/rho:rev:address/rho:asi:address/g' \
        "$file"
done

# 3. SPECIAL HANDLING FOR REGISTRY.RHO - Critical system file
echo "🏛️ Updating Registry.rho..."
if [ -f "casper/src/main/resources/Registry.rho" ]; then
    sed -i '' \
        -e 's/`rho:rchain:revVault`/`rho:rchain:asiVault`/g' \
        -e 's/`rho:rchain:multiSigRevVault`/`rho:rchain:multiSigASIVault`/g' \
        "casper/src/main/resources/Registry.rho"
    echo "✅ Updated Registry.rho system URIs"
fi

# 4. UPDATE SCALA PACKAGES - Critical for compilation
echo "📦 Updating Scala package declarations..."
find . -name "*.scala" | while read -r file; do
    sed -i '' \
        -e 's/package coop\.rchain\.node\.revvaultexport/package coop.rchain.node.asivaultexport/g' \
        -e 's/import coop\.rchain\.node\.revvaultexport/import coop.rchain.node.asivaultexport/g' \
        -e 's/import.*RevAddress/import coop.rchain.rholang.interpreter.util.ASIAddress/g' \
        "$file"
done

# 4b. ADDITIONAL: Update any remaining revvaultexport references in all file types
echo "🔄 Updating any remaining revvaultexport references..."
find . -type f \( -name "*.scala" -o -name "*.rs" -o -name "*.rho" \) ! -path "./.git/*" | while read -r file; do
    sed -i '' \
        -e 's/coop\.rchain\.node\.revvaultexport/coop.rchain.node.asivaultexport/g' \
        "$file"
done

# 5. UPDATE COMMENTS AND DOCUMENTATION (selective)
echo "📚 Updating comments..."
find . -type f \( -name "*.scala" -o -name "*.rs" -o -name "*.rho" \) ! -path "./.git/*" | while read -r file; do
    sed -i '' \
        -e 's/Rev vault/ASI vault/g' \
        -e 's/Rev address/ASI address/g' \
        -e 's/RevAddress for/ASIAddress for/g' \
        -e 's/RevAddresses/ASIAddresses/g' \
        -e 's/Get deployer.*rev address/Get deployer ASI address/g' \
        -e 's/Convert.*RevAddress/Convert into ASIAddress/g' \
        -e 's/correct RevAddress/correct ASIAddress/g' \
        -e 's/expecting RevAddress/expecting ASIAddress/g' \
        -e 's/all the revVault account/all the asiVault account/g' \
        -e 's/seedForRevVault/seedForASIVault/g' \
        "$file"
done

# 6. SPECIAL HANDLING FOR HARDCODED VALUES
echo "⚙️ Updating hardcoded values and constants..."
find . -name "*.scala" -o -name "*.rho" | while read -r file; do
    sed -i '' \
        -e 's/REV_ADDRESS_COUNT/ASI_ADDRESS_COUNT/g' \
        -e 's/revVaultPk/asiVaultPk/g' \
        -e 's/multiSigRevVaultPk/multiSigASIVaultPk/g' \
        -e 's/revGeneratorPk/asiGeneratorPk/g' \
        -e 's/revVaultPubKey/asiVaultPubKey/g' \
        -e 's/revVaultTimestamp/asiVaultTimestamp/g' \
        -e 's/multiSigRevVaultPk/multiSigASIVaultPk/g' \
        "$file"
done

# 7. RENAME DIRECTORIES FIRST
echo "📁 Renaming directories first..."

# Main revvaultexport directory
if [ -d "node/src/main/scala/coop/rchain/node/revvaultexport" ]; then
    git mv "node/src/main/scala/coop/rchain/node/revvaultexport" "node/src/main/scala/coop/rchain/node/asivaultexport"
    echo "✅ node/src/main/.../revvaultexport/ -> asivaultexport/"
fi

# Test revvaultexport directory - CRITICAL: This was missed before!
if [ -d "node/src/test/scala/coop/rchain/node/revvaultexport" ]; then
    git mv "node/src/test/scala/coop/rchain/node/revvaultexport" "node/src/test/scala/coop/rchain/node/asivaultexport"
    echo "✅ node/src/test/.../revvaultexport/ -> asivaultexport/"
fi

# Look for any other revvaultexport directories we might have missed
find . -type d -name "*revvaultexport*" -not -path "./.git/*" | while read -r dir; do
    if [ -d "$dir" ]; then
        newdir=$(echo "$dir" | sed 's/revvaultexport/asivaultexport/g')
        # Avoid creating nested directories
        if [[ "$newdir" != *"asivaultexport/asivaultexport"* ]]; then
            git mv "$dir" "$newdir"
            echo "✅ Found and moved: $dir -> $newdir"
        else
            echo "⚠️  Skipping to avoid nested directory: $dir"
        fi
    fi
done

# 8. RENAME FILES
echo "📁 Renaming files..."

# Main contracts
if [ -f "casper/src/main/resources/RevVault.rho" ]; then
    git mv "casper/src/main/resources/RevVault.rho" "casper/src/main/resources/ASIVault.rho"
    echo "✅ RevVault.rho -> ASIVault.rho"
fi

if [ -f "casper/src/main/resources/MultiSigRevVault.rho" ]; then
    git mv "casper/src/main/resources/MultiSigRevVault.rho" "casper/src/main/resources/MultiSigASIVault.rho"
    echo "✅ MultiSigRevVault.rho -> MultiSigASIVault.rho"
fi

# Tests
if [ -f "casper/src/test/resources/RevVaultTest.rho" ]; then
    git mv "casper/src/test/resources/RevVaultTest.rho" "casper/src/test/resources/ASIVaultTest.rho"
    echo "✅ RevVaultTest.rho -> ASIVaultTest.rho"
fi

if [ -f "casper/src/test/resources/MultiSigRevVaultTest.rho" ]; then
    git mv "casper/src/test/resources/MultiSigRevVaultTest.rho" "casper/src/test/resources/MultiSigASIVaultTest.rho"
    echo "✅ MultiSigRevVaultTest.rho -> MultiSigASIVaultTest.rho"
fi

# CRITICAL: RevAddressTest.rho exists and must be renamed
if [ -f "casper/src/test/resources/RevAddressTest.rho" ]; then
    git mv "casper/src/test/resources/RevAddressTest.rho" "casper/src/test/resources/ASIAddressTest.rho"
    echo "✅ RevAddressTest.rho -> ASIAddressTest.rho"
fi

# Rust files (may not exist)
if [ -f "rholang/src/rust/interpreter/util/rev_address.rs" ]; then
    git mv "rholang/src/rust/interpreter/util/rev_address.rs" "rholang/src/rust/interpreter/util/asi_address.rs"
    echo "✅ rev_address.rs -> asi_address.rs"
fi

if [ -f "casper/src/rust/genesis/contracts/rev_generator.rs" ]; then
    git mv "casper/src/rust/genesis/contracts/rev_generator.rs" "casper/src/rust/genesis/contracts/asi_generator.rs"
    echo "✅ rev_generator.rs -> asi_generator.rs"
fi

# Scala files
if [ -f "casper/src/main/scala/coop/rchain/casper/genesis/contracts/RevGenerator.scala" ]; then
    git mv "casper/src/main/scala/coop/rchain/casper/genesis/contracts/RevGenerator.scala" "casper/src/main/scala/coop/rchain/casper/genesis/contracts/ASIGenerator.scala"
    echo "✅ RevGenerator.scala -> ASIGenerator.scala"
fi

# CRITICAL: RevAddress.scala exists and must be renamed  
if [ -f "rholang/src/main/scala/coop/rchain/rholang/interpreter/util/RevAddress.scala" ]; then
    git mv "rholang/src/main/scala/coop/rchain/rholang/interpreter/util/RevAddress.scala" "rholang/src/main/scala/coop/rchain/rholang/interpreter/util/ASIAddress.scala"
    echo "✅ RevAddress.scala -> ASIAddress.scala"
fi

# Test specs
if [ -f "rholang/src/test/scala/coop/rchain/rholang/interpreter/util/RevAddressSpec.scala" ]; then
    git mv "rholang/src/test/scala/coop/rchain/rholang/interpreter/util/RevAddressSpec.scala" "rholang/src/test/scala/coop/rchain/rholang/interpreter/util/ASIAddressSpec.scala"
    echo "✅ RevAddressSpec.scala -> ASIAddressSpec.scala"
fi

if [ -f "casper/src/test/scala/coop/rchain/casper/genesis/contracts/RevAddressSpec.scala" ]; then
    git mv "casper/src/test/scala/coop/rchain/casper/genesis/contracts/RevAddressSpec.scala" "casper/src/test/scala/coop/rchain/casper/genesis/contracts/ASIAddressSpec.scala"
    echo "✅ RevAddressSpec.scala -> ASIAddressSpec.scala"
fi

# 9. UPDATE IMPORTS/MODULES - Final pass
echo "🔧 Final import updates..."
find . -name "*.scala" -o -name "*.rs" | while read -r file; do
    sed -i '' \
        -e 's/use.*rev_address/use crate::interpreter::util::asi_address/g' \
        -e 's/mod rev_address/mod asi_address/g' \
        "$file"
done

# 9b. ADDITIONAL: Final cleanup of any remaining references
echo "🧹 Final cleanup of remaining references..."
find . -type f \( -name "*.scala" -o -name "*.rs" -o -name "*.rho" \) ! -path "./.git/*" | while read -r file; do
    sed -i '' \
        -e 's/coop\.rchain\.node\.revvaultexport/coop.rchain.node.asivaultexport/g' \
        -e 's/node\.revvaultexport/node.asivaultexport/g' \
        "$file"
done

echo ""
echo "🎉 REV -> ASI migration completed!"
echo ""
echo "🔄 EXISTING BLOCKCHAIN MIGRATION SUMMARY:"
echo ""
echo "✅ Your blockchain infrastructure now supports ASI tokens:"
echo "   🔧 All code now uses ASI contracts and addresses"
echo "   🔗 System URIs changed: rho:rev:address → rho:asi:address"
echo "   🪙 New operations will create and use ASI tokens"
echo "   📊 APIs and UIs will show ASI instead of REV"
echo ""
echo "⚠️  CRITICAL - Impact on existing REV tokens:"
echo "   🚨 Existing REV tokens may become inaccessible!"
echo "   🔒 Old REV addresses use different URI (rho:rev:address)"
echo "   📱 Existing wallets may need updates to work with ASI"
echo "   🔄 Consider if you need REV→ASI migration mechanism"
echo ""
echo "📋 What was done:"
echo "   ✅ Replaced all identifiers (RevVault -> ASIVault etc.)"
echo "   ✅ Updated Registry URIs (rho:rchain:revVault -> rho:rchain:asiVault)"
echo "   ✅ Updated Registry.rho system file"
echo "   ✅ Updated Scala package declarations (ALL files)"
echo "   ✅ Updated method names (receiveRev -> receiveASI)"
echo "   ✅ Updated hardcoded constants and values"
echo "   ✅ Renamed files (.rho, .rs, .scala)"
echo "   ✅ Renamed directories and test specs"
echo "   ✅ Moved ALL revvaultexport directories (main AND test)"
echo "   ✅ Updated imports and modules (comprehensive)"
echo "   ✅ Updated comments in code files"
echo "   ✅ Final cleanup of all remaining references"
echo ""
echo "📝 What was NOT changed:"
echo "   ⏭️  Documentation files (.md, .txt, README, etc.) - ASI team should update these"
echo "      based on their specific features and requirements"
echo "   ⏭️  Historical wallet files (wallets_REV_BLOCK-*.txt) - preserved as blockchain history"
echo "   ⏭️  Hardcoded blockchain addresses - preserved as blockchain history"
echo "   ⏭️  Existing REV token balances in blockchain state - may need migration strategy"
echo ""
echo "🚀 IMMEDIATE NEXT STEPS:"
echo "   1. Check compilation: sbt compile"
echo "   2. Run tests: sbt test"
echo "   3. Test on development/testnet first"
echo "   4. Verify new ASI operations work correctly"
echo ""
echo "🤔 CONSIDER FOR PRODUCTION:"
echo "   ⚠️  How will existing REV holders access their tokens?"
echo "   ⚠️  Do you need backward compatibility contracts?"
echo "   ⚠️  Should you create REV→ASI conversion mechanism?"
echo "   ⚠️  How will you communicate changes to users?"
echo ""
echo "💡 Result: Your blockchain now operates with ASI tokens instead of REV!"
echo "   New users will have ASI addresses and ASI balances."