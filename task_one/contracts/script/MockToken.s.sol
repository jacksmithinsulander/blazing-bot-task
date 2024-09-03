// SPDX-License-Identifier: AGPL-3.0
pragma solidity 0.8.26;

import {Script, console} from "forge-std/Script.sol";
import {MockToken} from "test/mock/MockToken.sol";

contract TokenScript is Script {
    MockToken token;

    function run() external {
        vm.startBroadcast(vm.envUint("PKEY_ONE"));
        token = new MockToken("MyToken", "MTN");
        token.mint(vm.addr(vm.envUint("PKEY_ONE")), 10_000e18);
        vm.stopBroadcast();
    }
}