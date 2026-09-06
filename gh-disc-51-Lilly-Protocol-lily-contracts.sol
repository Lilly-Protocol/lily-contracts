// SPDX-License-Identifier: AGPL-3.0
pragma solidity ^0.8.20;

import "@openzeppelin/contracts/utils/Context.sol";

contract Config is Context {
    mapping(bytes32 => bytes32) private _config;

    event ConfigSet(bytes32 key, bytes32 value);

    function getConfigValue(bytes32 key) public view returns (bytes32) {
        return _config[key];
    }

    function getConfigValueOrDefault(bytes32 key, bytes32 defaultValue) public view returns (bytes32) {
        bytes32 value = _config[key];
        return value == 0 ? defaultValue : value;
    }

    function _setConfig(bytes32 key, bytes32 value) internal {
        _config[key] = value;
        emit ConfigSet(key, value);
    }
}