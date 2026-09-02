// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import "@openzeppelin/contracts/access/Ownable.sol";
import "@openzeppelin/contracts/utils/Address.sol";
import "@openzeppelin/contracts/utils/Context.sol";

contract Identity is Ownable {
    using Address for address;

    event IdentityCreated(address indexed owner, string name);
    event IdentityUpdated(address indexed owner, string name);
    event IdentityRevoked(address indexed owner);

    string private _name;
    address private _owner;

    constructor(string memory name_) {
        _name = name_;
        _owner = msg.sender;
        emit IdentityCreated(msg.sender, name_);
    }

    function updateIdentity(string memory name_) external onlyOwner {
        _name = name_;
        emit IdentityUpdated(msg.sender, name_);
    }

    function revokeIdentity() external onlyOwner {
        emit IdentityRevoked(msg.sender);
        selfdestruct(payable(owner()));
    }

    function name() external view returns (string memory) {
        return _name;
    }

    function owner() public view override returns (address) {
        return _owner;
    }
}

contract Wallet is Ownable {
    using Address for address;

    event FundsDeposited(address indexed sender, uint256 amount);
    event FundsWithdrawn(address indexed receiver, uint256 amount);
    event WalletTransferred(address indexed newOwner);

    mapping(address => uint256) private _balances;

    function deposit() external payable {
        require(msg.value > 0, "Wallet: deposit zero value");
        _balances[msg.sender] += msg.value;
        emit FundsDeposited(msg.sender, msg.value);
    }

    function withdraw(uint256 amount) external {
        require(_balances[msg.sender] >= amount, "Wallet: insufficient balance");
        _balances[msg.sender] -= amount;
        payable(msg.sender).sendValue(amount);
        emit FundsWithdrawn(msg.sender, amount);
    }

    function transferOwnership(address newOwner) public override onlyOwner {
        require(newOwner != address(0), "Wallet: zero address");
        super.transferOwnership(newOwner);
        emit WalletTransferred(newOwner);
    }

    function balanceOf(address account) external view returns (uint256) {
        return _balances[account];
    }
}