// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

import "@openzeppelin/contracts/access/Ownable.sol";
import "@openzeppelin/contracts/security/ReentrancyGuard.sol";

contract SpendLimitManager is Ownable, ReentrancyGuard {
    mapping(address => uint256) public spendLimits;

    event SpendLimitUpdated(address indexed account, uint256 newLimit);

    function updateSpendLimit(address account, uint256 newLimit) external onlyOwner {
        require(newLimit > 0, "Spend limit must be positive");
        spendLimits[account] = newLimit;
        emit SpendLimitUpdated(account, newLimit);
    }
}