use alloy::sol;

sol! {
    #[sol(rpc)]
    contract IMorpho {
        /// Get market state
        function market(bytes32 id) external view returns (
            uint128 totalSupplyAssets,
            uint128 totalSupplyShares,
            uint128 totalBorrowAssets,
            uint128 totalBorrowShares,
            uint128 lastUpdate,
            uint128 fee
        );

        /// Get a user's position in a market
        function position(bytes32 id, address user) external view returns (
            uint256 supplyShares,
            uint128 borrowShares,
            uint128 collateral
        );

        /// Get market parameters by ID
        function idToMarketParams(bytes32 id) external view returns (
            address loanToken,
            address collateralToken,
            address oracle,
            address irm,
            uint256 lltv
        );
    }

    #[sol(rpc)]
    contract IMorphoOracle {
        /// Returns price of 1 collateral base unit in loan base units, scaled by 1e36
        function price() external view returns (uint256);
    }

    /// Our liquidator contract interface
    #[sol(rpc)]
    contract IMorphoLiquidator {
        function liquidate(
            address loanToken,
            address collateralToken,
            address oracle,
            address irm,
            uint256 lltv,
            address borrower,
            uint256 seizedAssets,
            uint256 minProfit,
            bytes calldata swapData,
            address swapRouter
        ) external;

        function setFeeTier(address tokenA, address tokenB, uint24 fee) external;
        function setApprovedRouter(address router, bool approved) external;
        function rescueTokens(address token) external;
    }
}
