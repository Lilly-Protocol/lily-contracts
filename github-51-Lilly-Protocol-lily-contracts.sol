// In contracts/LilyConfig.sol

function getConfig(address key) external view returns (bytes memory value) {
    bytes32 key32 = bytes32(uint256(uint160(key)));
    value = _configs[key32];
    require(value.length > 0, "Config: key not found");
}

function getConfigBytes32(bytes32 key) external view returns (bytes memory value) {
    value = _configs[key];
    require(value.length > 0, "Config: key not found");
}

function getConfigUint(uint256 key) external view returns (uint256 value) {
    bytes32 key32 = bytes32(key);
    bytes memory data = _configs[key32];
    require(data.length > 0, "Config: key not found");
    require(data.length == 32, "Config: value not uint256");
    assembly {
        value := mload(add(data, 32))
    }
}

function getConfigBool(bytes32 key) external view returns (bool value) {
    bytes32 data = _configs[key];
    require(data != 0, "Config: key not found");
    require(data == 0x01 || data == 0x00, "Config: value not bool");
    return bool(data);
}