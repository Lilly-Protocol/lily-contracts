// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import "@openzeppelin/contracts/access/Ownable.sol";
import "@openzeppelin/contracts/utils/ReentrancyGuard.sol";

contract Payments is Ownable, ReentrancyGuard {
    // Fee parameters
    uint256 public constant FEE_BASIS_POINTS = 30; // 0.3% fee
    uint256 public constant MIN_FEE = 1000000000000000; // 0.001 ETH (in wei)
    uint256 public constant MAX_FEE = 50000000000000000; // 0.05 ETH (in wei)

    /**
     * @notice Calculates the fee for a given send amount
     * @param amount The amount to send (in wei)
     * @return fee The calculated fee (in wei)
     */
    function getSendFeeQuote(uint256 amount) external view returns (uint256 fee) {
        // Calculate fee as basis points of amount
        fee = (amount * FEE_BASIS_POINTS) / 10000;
        
        // Enforce min/max fee bounds
        if (fee < MIN_FEE) {
            fee = MIN_FEE;
        } else if (fee > MAX_FEE) {
            fee = MAX_FEE;
        }
    }

    /**
     * @notice Calculates the total amount needed to send a specific receive amount (including fee)
     * @param receiveAmount The desired receive amount (in wei)
     * @return totalAmount The total amount to send (in wei, including fee)
     */
    function getSendTotalQuote(uint256 receiveAmount) external view returns (uint256 totalAmount) {
        // Fee is added on top of receive amount, but subject to min/max
        uint256 estimatedFee = (receiveAmount * FEE_BASIS_POINTS) / (10000 - FEE_BASIS_POINTS);
        
        if (estimatedFee < MIN_FEE) {
            totalAmount = receiveAmount + MIN_FEE;
        } else if (estimatedFee > MAX_FEE) {
            totalAmount = receiveAmount + MAX_FEE;
        } else {
            totalAmount = receiveAmount + estimatedFee;
        }
    }

    /**
     * @notice Calculates the receive amount after deducting the fee for a given send amount
     * @param sendAmount The amount to send (in wei)
     * @return receiveAmount The amount received after fee (in wei)
     */
    function getReceiveQuote(uint256 sendAmount) external view returns (uint256 receiveAmount) {
        uint256 fee = getSendFeeQuote(sendAmount);
        receiveAmount = sendAmount - fee;
    }
}