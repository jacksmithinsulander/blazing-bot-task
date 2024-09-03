// SPDX-License-Identifier: AGPL-3.0
pragma solidity 0.8.26;

/**
 * @title ContractCheck
 * @author https://x.com/0xjsieth
 * @notice Helper contract to globally check if an address is a contract
 *
 */
library ContractCheck {
    /**
     * @notice
     *  Allows contract to check if the Token address actually is a contract
     *
     * @param _address address we want to  check
     *
     * @return _isAddressContract returns true if token is a contract, otherwise returns false
     *
     */
    function isContract(address _address) internal view returns (bool _isAddressContract) {
        uint256 size;

        assembly {
            size := extcodesize(_address)
        }

        _isAddressContract = size > 0;
    }
}