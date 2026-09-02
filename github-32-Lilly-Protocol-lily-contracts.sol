// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import "forge-std/Test.sol";
import "../src/LilyProtocol.sol";

contract LilyProtocolTest is Test {
    LilyProtocol public lily;

    function setUp() public {
        lily = new LilyProtocol();
    }

    function test_NextIntentIdIncrementsAcrossMultipleCreates() public {
        uint256 initialNextIntentId = lily.nextIntentId();
        
        // Create first intent
        lily.createIntent("intent1", "description1", 100);
        assertEq(lily.nextIntentId(), initialNextIntentId + 1, "NextIntentId should increment by 1 after first create");
        
        // Create second intent
        lily.createIntent("intent2", "description2", 200);
        assertEq(lily.nextIntentId(), initialNextIntentId + 2, "NextIntentId should increment by 1 after second create");
        
        // Create third intent
        lily.createIntent("intent3", "description3", 300);
        assertEq(lily.nextIntentId(), initialNextIntentId + 3, "NextIntentId should increment by 1 after third create");
    }
}