# Testing standards
Clean code requires clean tests. Therefore, the following standards and conventions should be applied when writing test code.

## Test locations

The unit test should be located in the `tests` folder, next to the source code of the given pallet/module.

The integration tests should be located in the `integration-tests` folder.

## Test file names

The test file names should be named according to the public API to be tested.

> Avoid using generic file names such as “tests.rs” or "lib.rs"

## Test names

The tests should follow the `ShouldWhen` naming template

Template:

```
<something>_should_<expected_behavior>_when_<state_under_test>
```

Examples:

```
create_yield_far_should_work_when_amm_pool_exist_for_asset_pair()
```

```
create_yield_farm_should_return_error_when_amm_pool_does_not_exist()
```

## Test structure
- Arrange-Act-Assert (AAA) pattern should be used to structure tests;
- Each part should be annotated with the `//Arrange`, `//Act` and `//Assert` comments;
- Builder pattern is encouraged for the `Arrange` part;
- Strive for testing one behavior per automated test;
    - Note that there can be multiple code assertions for one behavior

## Test targets
Strive for writing tests only for the public (or internal public) APIs, such as extrinsics.

Exceptions:
- logic-heavy internal functions where we treat them as separate units
- business logics where we apply property-based testing

## Helper libraries to be used

**PrettyAssertions**
- Prettifying the error log in case of test failure
- [Link to the crate](https://crates.io/crates/pretty-assertions)

**Test-case**
- Writing parameterized test
- Handy for removing test logic duplications and targeting some internal functions
- [Link to the crate](https://crates.io/crates/test-case)

**Proptest**
- Property based testing
- [Link to the crate](https://github.com/AltSysrq/proptest)