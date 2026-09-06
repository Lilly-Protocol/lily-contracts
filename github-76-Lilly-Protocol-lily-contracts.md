# Testing Risks and Mitigations

## `mock_all_auths` Test Risk

The `mock_all_auths` function in `test/Contract.t.sol` (or similar test file) mocks *all* authorization checks, which may lead to:

- **False positives**: Tests pass even if auth logic is broken elsewhere  
- **Masked vulnerabilities**: Critical access control flaws go undetected  
- **Over-mocking**: Reduces test fidelity to production behavior  

### Mitigation

- Prefer targeted mocking (e.g., `mock_auth(msg.sender, true)` for specific roles)  
- Add explicit tests for unauthorized access paths  
- Include integration tests that verify auth enforcement end-to-end  
- Consider using `vm.assume(!msg.sender == owner)` to prevent accidental owner impersonation in tests  

> 📝 *Reference: [Issue #76](https://github.com/Lilly-Protocol/lily-contracts/issues/76), 2026-08-25*