// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

interface IPoolAddressesProvider {
    /// @notice Returns the address of the Pool proxy
    function getPool() external view returns (address);

    /// @notice Returns the address of the PriceOracle proxy
    function getPriceOracle() external view returns (address);
}
