// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import "@openzeppelin/contracts/access/Ownable.sol";
import "@openzeppelin/contracts/utils/ReentrancyGuard.sol";

contract PaymentsMatching is Ownable, ReentrancyGuard {
    // Configuration structure
    struct Config {
        uint256 protocolFeeBPS;      // Protocol fee in basis points (e.g., 25 = 0.25%)
        uint256 maxFeeBPS;           // Maximum allowed fee in basis points
        uint256 minMatchAmount;      // Minimum amount required for a match
        bool enabled;                // Whether the protocol is enabled
    }

    // Storage
    Config public config;

    // Events
    event ConfigUpdated(
        uint256 protocolFeeBPS,
        uint256 maxFeeBPS,
        uint256 minMatchAmount,
        bool enabled
    );

    // Constructor
    constructor() {
        // Initialize with default config
        config = Config({
            protocolFeeBPS: 25,   // 0.25%
            maxFeeBPS: 100,       // 1.0%
            minMatchAmount: 1e15, // 0.001 ETH (assuming 18 decimals)
            enabled: true
        });
    }

    /**
     * @notice Updates the protocol configuration
     * @dev Only callable by owner
     * @param _protocolFeeBPS New protocol fee in basis points (max 10,000)
     * @param _maxFeeBPS New maximum fee in basis points (must be >= protocol fee)
     * @param _minMatchAmount New minimum match amount
     * @param _enabled Whether the protocol should be enabled
     */
    function updateConfig(
        uint256 _protocolFeeBPS,
        uint256 _maxFeeBPS,
        uint256 _minMatchAmount,
        bool _enabled
    ) external onlyOwner {
        // Validate inputs
        require(_protocolFeeBPS <= 10000, "Protocol: fee too high");
        require(_maxFeeBPS >= _protocolFeeBPS, "Protocol: max fee must be >= protocol fee");
        require(_maxFeeBPS <= 10000, "Protocol: max fee too high");

        // Update config
        config.protocolFeeBPS = _protocolFeeBPS;
        config.maxFeeBPS = _maxFeeBPS;
        config.minMatchAmount = _minMatchAmount;
        config.enabled = _enabled;

        // Emit event
        emit ConfigUpdated(_protocolFeeBPS, _maxFeeBPS, _minMatchAmount, _enabled);
    }

    /**
     * @notice Updates only the protocol fee (convenience function)
     * @param _protocolFeeBPS New protocol fee in basis points
     */
    function updateProtocolFee(uint256 _protocolFeeBPS) external onlyOwner {
        updateConfig(
            _protocolFeeBPS,
            config.maxFeeBPS,
            config.minMatchAmount,
            config.enabled
        );
    }

    /**
     * @notice Updates only the maximum fee (convenience function)
     * @param _maxFeeBPS New maximum fee in basis points
     */
    function updateMaxFee(uint256 _maxFeeBPS) external onlyOwner {
        updateConfig(
            config.protocolFeeBPS,
            _maxFeeBPS,
            config.minMatchAmount,
            config.enabled
        );
    }

    /**
     * @notice Updates only the minimum match amount (convenience function)
     * @param _minMatchAmount New minimum match amount
     */
    function updateMinMatchAmount(uint256 _minMatchAmount) external onlyOwner {
        updateConfig(
            config.protocolFeeBPS,
            config.maxFeeBPS,
            _minMatchAmount,
            config.enabled
        );
    }

    /**
     * @notice Enables/disables the protocol (convenience function)
     * @param _enabled Whether the protocol should be enabled
     */
    function setProtocolEnabled(bool _enabled) external onlyOwner {
        updateConfig(
            config.protocolFeeBPS,
            config.maxFeeBPS,
            config.minMatchAmount,
            _enabled
        );
    }
}