## [Unreleased]

### Added
- **PeTTa Integration**: MeTTa smart contract execution via SWI-Prolog
  - New system contract URN `rho:petta:execute` for executing MeTTa code from Rholang
  - Core execution function `petta_execute()` with 10-second timeout protection
  - Example contracts in `rholang/examples/system-contract/swipl/` demonstrating pattern matching and recursion
  - Full documentation:
    - URN specification: `rholang/docs/PETTA_URN_SPECIFICATION.md`
    - Test documentation: `rholang/tests/PETTA_TESTS_SUMMARY.md` and `TIMEOUT_TESTS.md`
    - Example README: `rholang/examples/system-contract/swipl/README.md`
    - API documentation via doc comments on all public items

## [v0.1.0-SNAPSHOT] - 2023-05-15
- Added new features
- Fixed bugs
- Improved performance
      

