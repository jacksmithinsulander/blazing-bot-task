// SPDX-License-Identifier: MIT
pragma solidity 0.8.26;

/* Openzeppelin Contract */
import {ERC20} from "@openzeppelin/contracts/token/ERC20/ERC20.sol";

contract MockToken is ERC20 {
    constructor(string memory _name, string memory _symbol) ERC20(_name, _symbol) {}

    function mint(address _user, uint256 _quantity) external {
        _mint(_user, _quantity);
    }
}