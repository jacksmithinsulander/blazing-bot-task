// SPDX-License-Identifier: AGPL-3.0
pragma solidity 0.8.26;

import "forge-std/Test.sol";

/* Blaze Contracts */
import {Blaze} from "src/Blaze.sol";

/* OpenZeppelin Interfaces */
import {IERC20} from "@openzeppelin/contracts/token/ERC20/IERC20.sol";

/* Mock Contracts */
import {MockToken} from "test/mock/MockToken.sol";

contract BlazeTest is Test {
    Blaze blaze;

    address alice;
    address bob;
    address tom;

    address[3] gathered;

    MockToken tokenA;
    MockToken tokenB;

    function setUp() external {
        blaze = new Blaze();

        alice = address(0x69);
        bob = address(0x42);
        tom = address(0x1337);

        gathered[0] = alice;
        gathered[1] = bob;
        gathered[2] = tom;

        tokenA = new MockToken("my cool token", "mct");
        tokenB = new MockToken("my fun token", "mft");

        bool ok;

        for (uint256 i = 0; i < 3; i++) {
            tokenA.mint(gathered[i], 100e18);
            (ok,) = payable(gathered[i]).call{value: 1 ether}("");
        }
    }
}