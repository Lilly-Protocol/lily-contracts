// contracts/LilyIntent.sol (example; adjust path/name as needed)
function create_intent(
    address recipient,
    uint256 amount,
    string calldata memo,
    uint256 deadline
) external {
    require(bytes(memo).length > 0, "Memo cannot be empty");
    // ... rest of existing logic
}