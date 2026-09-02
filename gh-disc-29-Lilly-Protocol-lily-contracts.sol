// contracts/LilyGovernance.sol (add to update_spend_limit function)
function update_spend_limit(address user, uint256 newLimit) external onlyGovernance {
    require(newLimit > 0, "Spend limit must be positive");
    spendLimits[user] = newLimit;
    emit SpendLimitUpdated(user, newLimit);
}