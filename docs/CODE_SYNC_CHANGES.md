# Code Sync Changes - Pending Updates

This document tracks the name changes and rebranding updates that should be applied when the codebase is ready for synchronization.

## Overview

These changes were identified during PR #96 review as being premature - the code examples and references need to be updated in the actual codebase before these documentation changes can be applied.

## Files Requiring Updates When Code is Ready

### 1. DEVELOPER.md
**Changes to Apply:**
- Replace all instances of `sbt:rchain>` with `sbt:rnode>`
- Update repository paths from `/rchain/` to `/rnode/`
- Update Docker image names from `coop.rchain/rnode` to `coop.f1r3fly/rnode`
- Update Wiki links from `github.com/rchain/rchain` to `github.com/F1R3FLY-io/rnode`
- Update network operation descriptions from "RChain" to "RNode"

### 2. bitcoin-anchor/README.md
**Changes to Apply:**
- Update all code examples to use new F1R3FLY naming conventions
- Ensure all import statements and function names match the rebranded codebase
- Update any RChain references to RNode/F1R3FLY

### 3. comm/README.md
**Changes to Apply:**
- Change `F1r3flyClientCertVerifier` to `RNodeClientCertVerifier` once this class exists in the codebase
- Update all network-related class names to match the new naming convention

### 4. rspace/README.md
**Changes to Apply:**
- Update URL from `http://rchain-architecture.readthedocs.io` to `http://rnode-architecture.readthedocs.io` (when available)
- Update GitHub repository link from `github.com/rchain/rchain` to `github.com/F1R3FLY-io/rnode` (when repository is renamed/moved)
- Change "RChain Tuple Space" references as appropriate

## Implementation Notes

1. **Prerequisites**: Before applying these changes, ensure:
   - The actual Scala/Rust code has been updated with new class names
   - Repository URLs and documentation sites have been updated
   - Docker image names have been changed in the build configuration

2. **Testing**: After applying these changes:
   - Verify all code examples compile and run correctly
   - Test all documentation links are valid
   - Ensure Docker builds use correct image names

3. **Coordination Required**:
   - Coordinate with DevOps for Docker registry updates
   - Ensure CI/CD pipelines are updated with new naming
   - Update any external documentation or wiki pages

## Related Pull Requests

- PR #96: Initial documentation updates (reverted premature changes)
- This branch (rgb-code-sync-updates): Holds the changes for future application

## Checklist for Future Application

- [ ] Verify all code classes/functions have been renamed
- [ ] Update build configuration files (build.sbt, Cargo.toml)
- [ ] Update Docker configurations
- [ ] Apply documentation changes from this branch
- [ ] Test all examples and commands
- [ ] Update external documentation links
- [ ] Merge this branch after code sync is complete