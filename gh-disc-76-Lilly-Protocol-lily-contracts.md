# Testing Risks and Mitigations

## `mock_all_auths` Test Function

**Risk**: The `mock_all_auths` function grants all roles/permissions to the caller during testing, which may mask security issues such as:
- Missing access control checks in edge cases
- Accidental privilege escalation if test setup leaks into production (e.g., via misconfigured deployment)
- Overly permissive test scenarios that fail to validate least-privilege enforcement

**Mitigation**:
- Always verify that critical functions are tested with *minimal required permissions* in addition to `mock_all_auths` tests
- Include explicit negative tests (e.g., `test_reverts_without_role`) for sensitive operations
- Never use `mock_all_auths` in integration or fuzzing setups unless explicitly intended for exhaustive coverage with expected failures
- Consider adding a `require(!isProduction, "...")` guard if this mock is ever used in non-test environments (though it should only exist in test files)