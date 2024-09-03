// SPDX-License-Identifier: AGPL-3.0
pragma solidity 0.8.26;

import {Script, console} from "forge-std/Script.sol";
import {Blaze} from "src/Blaze.sol";

contract BlazeScript is Script {
    Blaze blaze;

    function run() external {
        vm.startBroadcast(vm.envUint("PKEY_ONE"));
        blaze = new Blaze();
        vm.stopBroadcast();
    }
}
