// SPDX-License-Identifier: AGPL-3.0
pragma solidity 0.8.26;

/* OpenZeppelin Interfaces */
import {IERC20} from "@openzeppelin/contracts/token/ERC20/IERC20.sol";

/* OpenZeppelin Libraries */
import "@openzeppelin/contracts/utils/ReentrancyGuard.sol";

/* 0xjsi.eth Libraries */
import {ContractCheck} from "src/libraries/ContractCheck.sol";
import {Errors} from "src/libraries/Errors.sol";
import {DataTypes} from "src/libraries/DataTypes.sol";

/**
 * @title Blaze
 * @author https://x.com/0xjsieth
 * @notice hiring task for blazebot
 *
 */
contract Blaze is ReentrancyGuard {
    using ContractCheck for address;

    function disperseETH(address[] calldata _receivers) external payable returns(bool _success) {
        if (_receivers.length == 0) revert Errors.RECEIVERS_CANT_BE_EMPTY();

        uint256 amountPerReceiver = msg.value / _receivers.length;

        for (uint256 i = 0; i < _receivers.length; i++) {
            (_success,) = payable(_receivers[i]).call{value: amountPerReceiver}("");
            if (!_success) revert Errors.FAILED_TO_PAY(_receivers[i]);
        }
    }

    function disperseToken(address[] calldata _receivers, uint256 _amount, address _token) external returns(bool _success) {
        if (_receivers.length == 0) revert Errors.RECEIVERS_CANT_BE_EMPTY();

        if (!_token.isContract()) revert Errors.ADDRESS_NOT_A_CONTRACT();

        IERC20 tokenInstance = IERC20(_token);

        if (_amount > tokenInstance.balanceOf(msg.sender)) revert Errors.NOT_ENOUGH_HOLDINGS();

        uint256 amountPerReceiver = _amount / _receivers.length;

        for (uint256 i = 0; i < _receivers.length; i++) {
            _success = tokenInstance.transferFrom(msg.sender, _receivers[i], amountPerReceiver);
            if (!_success) revert Errors.FAILED_TO_PAY(_receivers[i]);
        }
    }

    function collectToken(DataTypes.CollectData[] calldata _collectData, address _token, address _reciever) external returns(bool _success) {
        if (!_token.isContract()) revert Errors.ADDRESS_NOT_A_CONTRACT();

        if (_collectData.length < 1) revert Errors.COLLECT_DATA_CANT_BE_EMPTY();
  
        IERC20 tokenInstance = IERC20(_token);

        for (uint256 i = 0; i < _collectData.length; i++) {
            _success = tokenInstance.transferFrom(_collectData[i].payee, _reciever, _collectData[i].amount);
            if (!_success) revert Errors.FAILED_TO_PAY(_reciever);
        }
    }

    function getTokenHolding(address _token, address _user) external view returns(uint256 _userHoldings) {
        if (!_token.isContract()) revert Errors.ADDRESS_NOT_A_CONTRACT();

        _userHoldings = IERC20(_token).balanceOf(_user);
    }

    function getEthHoldings(address _user) external view returns(uint256 _userHoldings) {
        _userHoldings = _user.balance;
    }
}
