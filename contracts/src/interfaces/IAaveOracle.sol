// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

interface IAaveOracle {
    /// @notice Returns the price of the asset in the base currency (USD, 8 decimals)
    /// @param asset The address of the asset
    /// @return The price of the asset
    function getAssetPrice(address asset) external view returns (uint256);

    /// @notice Sets or replaces price sources for a list of assets
    /// @param assets The addresses of the assets
    /// @param sources The addresses of the Chainlink price feed sources
    function setAssetSources(address[] calldata assets, address[] calldata sources) external;

    /// @notice Returns the list of prices for the given assets
    /// @param assets The addresses of the assets
    /// @return The prices of the assets
    function getAssetsPrices(address[] calldata assets) external view returns (uint256[] memory);
}
