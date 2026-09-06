// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import "@openzeppelin/contracts/utils/ReentrancyGuard.sol";
import "@openzeppelin/contracts/access/Ownable.sol";

contract IntentRegistry is ReentrancyGuard, Ownable {
    error MissingRecord();

    mapping(bytes32 => Intent) private _intents;

    struct Intent {
        address owner;
        uint256 createdAt;
        bytes data;
    }

    function get_intent(bytes32 id) external view returns (Intent memory) {
        if (_intents[id].owner == address(0)) {
            revert MissingRecord();
        }
        return _intents[id];
    }

    function set_intent(bytes32 id, address owner, bytes memory data) external onlyOwner {
        _intents[id] = Intent({
            owner: owner,
            createdAt: block.timestamp,
            data: data
        });
    }
}