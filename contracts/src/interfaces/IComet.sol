// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

/// @title IComet
/// @notice Minimal interface for Compound V3 (Comet) liquidation interactions
/// @dev Compound V3 uses a two-step liquidation process:
///      1. absorb() — protocol seizes underwater position's collateral
///      2. buyCollateral() — anyone can purchase absorbed collateral at a discount (2.5-5%)
interface IComet {
    // ═══════════════════════════════════════════════════════════════
    //                      LIQUIDATION
    // ═══════════════════════════════════════════════════════════════

    /// @notice Absorb underwater accounts, seizing their collateral into protocol reserves
    /// @param absorber The address that triggers the absorption (receives no reward, just gas cost)
    /// @param accounts Array of underwater accounts to absorb
    function absorb(address absorber, address[] calldata accounts) external;

    /// @notice Buy absorbed collateral at a discount using base token
    /// @param asset The collateral asset to purchase
    /// @param minAmount Minimum amount of collateral to receive (slippage protection)
    /// @param baseAmount Amount of base token to spend
    /// @param recipient Address to receive the purchased collateral
    function buyCollateral(address asset, uint minAmount, uint baseAmount, address recipient) external;

    // ═══════════════════════════════════════════════════════════════
    //                      VIEW FUNCTIONS
    // ═══════════════════════════════════════════════════════════════

    /// @notice Check if an account is liquidatable (health factor < 1)
    /// @param account The account to check
    /// @return True if the account can be absorbed
    function isLiquidatable(address account) external view returns (bool);

    /// @notice Quote how much collateral you'd receive for a given base amount
    /// @param asset The collateral asset
    /// @param baseAmount The amount of base token to spend
    /// @return The amount of collateral asset you would receive
    function quoteCollateral(address asset, uint baseAmount) external view returns (uint);

    /// @notice Get the protocol's total reserves (can be negative if in deficit)
    /// @return The signed reserve balance in base token units
    function getReserves() external view returns (int);

    /// @notice Get the target reserve level for the protocol
    /// @return The target reserves in base token units
    function targetReserves() external view returns (uint);

    /// @notice Get the amount of absorbed collateral available for purchase
    /// @param asset The collateral asset address
    /// @return The amount of collateral reserves available to buy
    function getCollateralReserves(address asset) external view returns (uint);

    /// @notice Get an account's collateral balance for a specific asset
    /// @param account The account address
    /// @param asset The collateral asset address
    /// @return The collateral balance
    function collateralBalanceOf(address account, address asset) external view returns (uint128);

    // ═══════════════════════════════════════════════════════════════
    //                      MARKET INFO
    // ═══════════════════════════════════════════════════════════════

    /// @notice The base token of this Comet market (e.g., USDC)
    function baseToken() external view returns (address);

    /// @notice The scale factor for the base token (10^decimals)
    function baseScale() external view returns (uint);

    /// @notice Number of collateral assets supported by this market
    function numAssets() external view returns (uint8);

    /// @notice Get full configuration info for a collateral asset by index
    /// @param i The asset index (0-based)
    /// @return offset The bit offset for the asset's storage
    /// @return asset The collateral asset address
    /// @return priceFeed The Chainlink price feed address for this asset
    /// @return scale The scale factor (10^decimals) for the asset
    /// @return borrowCollateralFactor The factor used for borrow limit calculations (1e18 = 100%)
    /// @return liquidateCollateralFactor The factor used for liquidation threshold (1e18 = 100%)
    /// @return liquidationFactor The penalty factor applied during liquidation (1e18 = 100%)
    /// @return supplyCap The maximum amount of this collateral that can be supplied
    function getAssetInfo(uint8 i)
        external
        view
        returns (
            uint8 offset,
            address asset,
            address priceFeed,
            uint64 scale,
            uint64 borrowCollateralFactor,
            uint64 liquidateCollateralFactor,
            uint64 liquidationFactor,
            uint128 supplyCap
        );

    /// @notice Get an account's borrow balance (debt owed in base token)
    /// @param account The account address
    /// @return The borrow balance in base token units
    function borrowBalanceOf(address account) external view returns (uint256);

    // ═══════════════════════════════════════════════════════════════
    //                      SUPPLY / WITHDRAW
    // ═══════════════════════════════════════════════════════════════

    /// @notice Supply an asset to the protocol (collateral or base token)
    /// @param asset The asset address to supply
    /// @param amount The amount to supply
    function supply(address asset, uint256 amount) external;

    /// @notice Withdraw an asset from the protocol (borrow base or withdraw collateral)
    /// @param asset The asset address to withdraw
    /// @param amount The amount to withdraw
    function withdraw(address asset, uint256 amount) external;
}
