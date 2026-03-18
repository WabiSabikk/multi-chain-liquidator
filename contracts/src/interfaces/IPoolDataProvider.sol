// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

interface IPoolDataProvider {
    /// @notice Returns the user reserve data
    /// @param asset The address of the underlying asset
    /// @param user The address of the user
    /// @return currentATokenBalance The current aToken balance
    /// @return currentStableDebt The current stable debt
    /// @return currentVariableDebt The current variable debt
    /// @return principalStableDebt The principal stable debt
    /// @return scaledVariableDebt The scaled variable debt
    /// @return stableBorrowRate The stable borrow rate
    /// @return liquidityRate The liquidity rate
    /// @return stableRateLastUpdated Timestamp of last stable rate update
    /// @return usageAsCollateralEnabled Whether the asset is used as collateral
    function getUserReserveData(address asset, address user)
        external
        view
        returns (
            uint256 currentATokenBalance,
            uint256 currentStableDebt,
            uint256 currentVariableDebt,
            uint256 principalStableDebt,
            uint256 scaledVariableDebt,
            uint256 stableBorrowRate,
            uint256 liquidityRate,
            uint40 stableRateLastUpdated,
            bool usageAsCollateralEnabled
        );

    /// @notice Returns the reserve configuration data
    /// @param asset The address of the underlying asset
    /// @return decimals The number of decimals
    /// @return ltv The loan-to-value (in basis points)
    /// @return liquidationThreshold The liquidation threshold (in basis points)
    /// @return liquidationBonus The liquidation bonus (in basis points, 10500 = 5% bonus)
    /// @return reserveFactor The reserve factor (in basis points)
    /// @return usageAsCollateralEnabled Whether the asset can be used as collateral
    /// @return borrowingEnabled Whether borrowing is enabled
    /// @return stableBorrowRateEnabled Whether stable rate borrowing is enabled
    /// @return isActive Whether the reserve is active
    /// @return isFrozen Whether the reserve is frozen
    function getReserveConfigurationData(address asset)
        external
        view
        returns (
            uint256 decimals,
            uint256 ltv,
            uint256 liquidationThreshold,
            uint256 liquidationBonus,
            uint256 reserveFactor,
            bool usageAsCollateralEnabled,
            bool borrowingEnabled,
            bool stableBorrowRateEnabled,
            bool isActive,
            bool isFrozen
        );
}
