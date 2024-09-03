// SPDX-License-Identifier: AGPL-3.0
pragma solidity 0.8.26;

import {Script, console} from "forge-std/Script.sol";

contract BlazeScript is Script {
    address deployer = vm.addr(vm.envUint("PKEY_TWO"));
    bool ok;

    function run() external {
        vm.startBroadcast(vm.envUint("PKEY_TWO"));
        (ok,) = payable(vm.addr(vm.envUint("PKEY_ONE"))).call{value: deployer.balance / 4}("");
        (ok,) = payable(vm.addr(vm.envUint("PKEY_THREE"))).call{value: deployer.balance / 4}("");
        (ok,) = payable(vm.addr(vm.envUint("PKEY_FOUR"))).call{value: deployer.balance / 4}("");
        vm.stopBroadcast();
    }
}