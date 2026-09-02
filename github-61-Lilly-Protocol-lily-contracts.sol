// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import "@openzeppelin/contracts/access/Ownable.sol";

contract Payments is Ownable {
    // Fee configuration
    uint256 public constant FEE_PRECISION = 10000; // 4 decimal places (e.g., 100 = 1%)
    uint256 public fixedFee; // in wei
    uint256 public variableFeeBasisPoints; // basis points (e.g., 250 = 2.5%)

    constructor(uint256 _fixedFee, uint256 _variableFeeBasisPoints) Ownable(msg.sender) {
        fixedFee = _fixedFee;
        variableFeeBasisPoints = _variableFeeBasisPoints;
    }

    /**
     * @notice Calculates the total fee for a given amount
     * @param amount The transaction amount in wei
     * @return totalFee The total fee in wei (fixed + variable)
     */
    function quoteFee(uint256 amount) external view returns (uint256 totalFee) {
        uint256 variableComponent = (amount * variableFeeBasisPoints) / FEE_PRECISION;
        totalFee = fixedFee + variableComponent;
    }

    /**
     * @notice Returns the breakdown of fees for a given amount
     * @param amount The transaction amount in wei
     * @return fixed The fixed fee component in wei
     * @return variable The variable fee component in wei
     */
    function getFeeBreakdown(uint256 amount) external view returns (uint256 fixed, uint256 variable) {
        fixed = fixedFee;
        variable = (amount * variableFeeBasisPoints) / FEE_PRECISION;
    }

    /**
     * @notice Calculates the net amount received after fees for a given gross amount
     * @param amount The gross transaction amount in wei
     * @return netAmount The amount after deducting fees in wei
     */
    function quoteNetAmount(uint256 amount) external view returns (uint256 netAmount) {
        uint256 totalFee = quoteFee(amount);
        require(amount >= totalFee, "Amount too low for fees");
        netAmount = amount - totalFee;
    }

    // Update fee configuration (owner-only)
    function setFees(uint256 _fixedFee, uint256 _variableFeeBasisPoints) external onlyOwner {
        fixedFee = _fixedFee;
        variableFeeBasisPoints = _variableFeeBasisPoints;
    }
}