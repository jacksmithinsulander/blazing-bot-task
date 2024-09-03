// SPDX-License-Identifier: AGPL-3.0
pragma solidity 0.8.26;

library Errors {
    error ADDRESS_NOT_A_CONTRACT();

    error FAILED_TO_PAY(address who);

    error NOT_ENOUGH_HOLDINGS();

    error RECEIVERS_CANT_BE_EMPTY();

    error COLLECT_DATA_CANT_BE_EMPTY();
}