// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

interface IPool {
    /// @notice Execute a flash loan (simplified single-asset version)
    /// @param receiverAddress The contract receiving the flash-loaned assets (must implement executeOperation)
    /// @param asset The address of the underlying asset to flash loan
    /// @param amount The amount to flash loan
    /// @param params Arbitrary bytes to pass to the receiver's executeOperation
    /// @param referralCode Referral code (use 0)
    function flashLoanSimple(
        address receiverAddress,
        address asset,
        uint256 amount,
        bytes calldata params,
        uint16 referralCode
    ) external;

    /// @notice Liquidate an undercollateralized position
    /// @param collateralAsset The address of the collateral asset to receive
    /// @param debtAsset The address of the debt asset to repay
    /// @param user The address of the borrower being liquidated
    /// @param debtToCover The amount of debt to repay
    /// @param receiveAToken True to receive aTokens, false to receive underlying
    function liquidationCall(
        address collateralAsset,
        address debtAsset,
        address user,
        uint256 debtToCover,
        bool receiveAToken
    ) external;

    /// @notice Returns the user account data across all reserves
    /// @param user The address of the user
    /// @return totalCollateralBase The total collateral in the base currency (USD, 8 decimals)
    /// @return totalDebtBase The total debt in the base currency
    /// @return availableBorrowsBase The borrowing power remaining
    /// @return currentLiquidationThreshold The weighted average liquidation threshold
    /// @return ltv The weighted average loan-to-value
    /// @return healthFactor The health factor (1e18 = 1.0)
    function getUserAccountData(address user)
        external
        view
        returns (
            uint256 totalCollateralBase,
            uint256 totalDebtBase,
            uint256 availableBorrowsBase,
            uint256 currentLiquidationThreshold,
            uint256 ltv,
            uint256 healthFactor
        );

    /// @notice Supply an asset to the pool
    /// @param asset The address of the underlying asset to supply
    /// @param amount The amount to supply
    /// @param onBehalfOf The address that will receive the aTokens
    /// @param referralCode Referral code (use 0)
    function supply(address asset, uint256 amount, address onBehalfOf, uint16 referralCode) external;

    /// @notice Borrow an asset from the pool
    /// @param asset The address of the underlying asset to borrow
    /// @param amount The amount to borrow
    /// @param interestRateMode 1 = stable, 2 = variable
    /// @param referralCode Referral code (use 0)
    /// @param onBehalfOf The address that will receive the debt tokens
    function borrow(address asset, uint256 amount, uint256 interestRateMode, uint16 referralCode, address onBehalfOf)
        external;
}
