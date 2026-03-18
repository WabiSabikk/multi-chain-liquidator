use alloy::sol;

sol! {
    #[sol(rpc)]
    contract IPool {
        function getUserAccountData(address user) external view returns (
            uint256 totalCollateralBase,
            uint256 totalDebtBase,
            uint256 availableBorrowsBase,
            uint256 currentLiquidationThreshold,
            uint256 ltv,
            uint256 healthFactor
        );

        function getUserEMode(address user) external view returns (uint256);

        function getEModeCategoryData(uint8 id) external view returns (
            uint16 ltv,
            uint16 liquidationThreshold,
            uint16 liquidationBonus,
            address priceSource,
            string memory label
        );
    }

    #[sol(rpc)]
    contract IPoolDataProvider {
        function getAllReservesTokens() external view returns (TokenData[] memory);

        struct TokenData {
            string symbol;
            address tokenAddress;
        }

        function getReserveConfigurationData(address asset) external view returns (
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

        function getUserReserveData(address asset, address user) external view returns (
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

        function getReserveEModeCategory(address asset) external view returns (uint256);
    }

    #[sol(rpc)]
    contract IAaveOracle {
        function getAssetsPrices(address[] memory assets) external view returns (uint256[] memory);
    }

    #[sol(rpc)]
    contract IMulticall3 {
        struct Call3 {
            address target;
            bool allowFailure;
            bytes callData;
        }

        struct Result {
            bool success;
            bytes returnData;
        }

        function aggregate3(Call3[] calldata calls) public payable returns (Result[] memory returnData);
    }

    #[sol(rpc)]
    contract IFlashLoanLiquidator {
        /// Simple overload (uses default Uniswap V3 swap path)
        function executeLiquidation(
            address collateralAsset,
            address debtAsset,
            address user,
            uint256 debtToCover,
            uint256 minProfit
        ) external;

        /// Full overload with custom DEX routing
        function executeLiquidation(
            address collateralAsset,
            address debtAsset,
            address user,
            uint256 debtToCover,
            uint256 minProfit,
            bytes calldata swapData,
            address swapRouter
        ) external;

        /// Set Uniswap V3 fee tier for a pair
        function setFeeTier(address tokenA, address tokenB, uint24 fee) external;

        /// Add/remove approved DEX router
        function setApprovedRouter(address router, bool approved) external;

        /// Cross-token flash loan liquidation (Lendle USDT debt via USDC flash loan)
        function executeCrossTokenLiquidation(
            address collateralAsset,
            address debtAsset,
            address user,
            uint256 debtToCover,
            address flashLoanAsset,
            uint256 flashLoanAmount,
            bytes calldata preSwapData,
            address preSwapRouter,
            bytes calldata postSwapData,
            address postSwapRouter
        ) external;
    }
}
