// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import "forge-std/Test.sol";
import "../src/BasisPoint.sol";

contract BasisPointTest is Test {
    using BasisPoint for uint256;

    function test_Bounds_Inclusive() public {
        // Valid bounds: 0 to 10,000 inclusive
        vm.assume(0 <= 10000);
        uint256 bp = bound(0, 0, 10000);
        assertTrue(bp.isValid(), "Valid basis point should be accepted");
        assertTrue(bp.toUint() == bp, "Conversion should preserve value");
    }

    function test_Bounds_Exclusive_Upper() public {
        // Values above 10,000 are invalid
        uint256 invalidBp = bound(10001, 10001, type(uint256).max);
        vm.assume(invalidBp > 10000);
        assertFalse(invalidBp.isValid(), "Basis point >10,000 should be invalid");
    }

    function test_Bounds_Exclusive_Lower() public {
        // Negative values not possible in uint256, but test 0 explicitly
        uint256 zeroBp = 0;
        assertTrue(zeroBp.isValid(), "Basis point 0 should be valid");
    }

    function test_Arithmetic_Safe_Addition() public {
        // Ensure addition doesn’t overflow or exceed 10,000
        uint256 bp1 = bound(0, 0, 5000);
        uint256 bp2 = bound(0, 0, 5000);
        vm.assume(bp1 + bp2 <= 10000);
        uint256 sum = bp1.addBasisPoints(bp2);
        assertTrue(sum.isValid(), "Sum of valid basis points should be valid");
        assertTrue(sum.toUint() == bp1 + bp2, "Sum should equal arithmetic addition");
    }

    function test_Arithmetic_Safe_Multiplication() public {
        // Test multiplication: (bp * value) / 10,000
        uint256 bp = bound(0, 0, 10000);
        uint256 value = bound(1, 1, 1e18);
        uint256 result = bp.multiplyBasisPoints(value);
        assertTrue(result <= value, "Result should not exceed original value");
    }

    function test_Idempotency_Conversion() public {
        uint256 bp = bound(0, 0, 10000);
        uint256 converted = bp.toUint();
        assertTrue(converted.isValid(), "Converted value should be valid");
        assertTrue(converted.toUint() == bp, "Double conversion should be idempotent");
    }

    function test_Division_By_Zero_Protection() public {
        // Multiplication by zero should be safe
        uint256 zeroBp = 0;
        uint256 value = bound(1, 1, 1e18);
        uint256 result = zeroBp.multiplyBasisPoints(value);
        assertTrue(result == 0, "Zero basis point should yield zero result");
    }
}