import { BN } from '@coral-xyz/anchor';
import {
	PerpMarketAccount,
	PositionDirection,
	MarginCategory,
	SpotMarketAccount,
	SpotBalanceType,
	MarketType,
} from '../types';
import {
	calculateAmmReservesAfterSwap,
	calculatePrice,
	calculateUpdatedAMMSpreadReserves,
	getSwapDirection,
	calculateUpdatedAMM,
	calculateMarketOpenBidAsk,
} from './amm';
import {
	calculateSizeDiscountAssetWeight,
	calculateSizePremiumLiabilityWeight,
} from './margin';
import { OraclePriceData } from '../oracles/types';
import {
	BASE_PRECISION,
	MARGIN_PRECISION,
	PRICE_TO_QUOTE_PRECISION,
	ZERO,
	QUOTE_SPOT_MARKET_INDEX,
} from '../constants/numericConstants';
import { getTokenAmount } from './spotBalance';
import { DLOB } from '../dlob/DLOB';
import { assert } from '../assert/assert';

/**
 * Calculates market mark price
 *
 * @param market
 * @return markPrice : Precision PRICE_PRECISION
 */
export function calculateReservePrice(
	market: PerpMarketAccount,
	oraclePriceData: OraclePriceData
): BN {
	const newAmm = calculateUpdatedAMM(market.amm, oraclePriceData);
	return calculatePrice(
		newAmm.baseAssetReserve,
		newAmm.quoteAssetReserve,
		newAmm.pegMultiplier
	);
}

/**
 * Calculates market bid price
 *
 * @param market
 * @return bidPrice : Precision PRICE_PRECISION
 */
export function calculateBidPrice(
	market: PerpMarketAccount,
	oraclePriceData: OraclePriceData
): BN {
	const { baseAssetReserve, quoteAssetReserve, newPeg } =
		calculateUpdatedAMMSpreadReserves(
			market.amm,
			PositionDirection.SHORT,
			oraclePriceData
		);

	return calculatePrice(baseAssetReserve, quoteAssetReserve, newPeg);
}

/**
 * Calculates market ask price
 *
 * @param market
 * @return askPrice : Precision PRICE_PRECISION
 */
export function calculateAskPrice(
	market: PerpMarketAccount,
	oraclePriceData: OraclePriceData
): BN {
	const { baseAssetReserve, quoteAssetReserve, newPeg } =
		calculateUpdatedAMMSpreadReserves(
			market.amm,
			PositionDirection.LONG,
			oraclePriceData
		);

	return calculatePrice(baseAssetReserve, quoteAssetReserve, newPeg);
}

export function calculateNewMarketAfterTrade(
	baseAssetAmount: BN,
	direction: PositionDirection,
	market: PerpMarketAccount
): PerpMarketAccount {
	const [newQuoteAssetReserve, newBaseAssetReserve] =
		calculateAmmReservesAfterSwap(
			market.amm,
			'base',
			baseAssetAmount.abs(),
			getSwapDirection('base', direction)
		);

	const newAmm = Object.assign({}, market.amm);
	const newMarket = Object.assign({}, market);
	newMarket.amm = newAmm;
	newMarket.amm.quoteAssetReserve = newQuoteAssetReserve;
	newMarket.amm.baseAssetReserve = newBaseAssetReserve;

	return newMarket;
}

export function calculateOracleReserveSpread(
	market: PerpMarketAccount,
	oraclePriceData: OraclePriceData
): BN {
	const reservePrice = calculateReservePrice(market, oraclePriceData);
	return calculateOracleSpread(reservePrice, oraclePriceData);
}

export function calculateOracleSpread(
	price: BN,
	oraclePriceData: OraclePriceData
): BN {
	return price.sub(oraclePriceData.price);
}

export function calculateMarketMarginRatio(
	market: PerpMarketAccount,
	size: BN,
	marginCategory: MarginCategory
): number {
	let marginRatio;
	switch (marginCategory) {
		case 'Initial': {
			marginRatio = calculateSizePremiumLiabilityWeight(
				size,
				new BN(market.imfFactor),
				new BN(market.marginRatioInitial),
				MARGIN_PRECISION
			).toNumber();
			break;
		}
		case 'Maintenance': {
			marginRatio = calculateSizePremiumLiabilityWeight(
				size,
				new BN(market.imfFactor),
				new BN(market.marginRatioMaintenance),
				MARGIN_PRECISION
			).toNumber();
			break;
		}
	}

	return marginRatio;
}

export function calculateUnrealizedAssetWeight(
	market: PerpMarketAccount,
	quoteSpotMarket: SpotMarketAccount,
	unrealizedPnl: BN,
	marginCategory: MarginCategory,
	oraclePriceData: OraclePriceData
): BN {
	let assetWeight: BN;
	switch (marginCategory) {
		case 'Initial':
			assetWeight = new BN(market.unrealizedPnlInitialAssetWeight);

			if (market.unrealizedPnlMaxImbalance.gt(ZERO)) {
				const netUnsettledPnl = calculateNetUserPnlImbalance(
					market,
					quoteSpotMarket,
					oraclePriceData
				);
				if (netUnsettledPnl.gt(market.unrealizedPnlMaxImbalance)) {
					assetWeight = assetWeight
						.mul(market.unrealizedPnlMaxImbalance)
						.div(netUnsettledPnl);
				}
			}

			assetWeight = calculateSizeDiscountAssetWeight(
				unrealizedPnl,
				new BN(market.unrealizedPnlImfFactor),
				assetWeight
			);
			break;
		case 'Maintenance':
			assetWeight = new BN(market.unrealizedPnlMaintenanceAssetWeight);
			break;
	}

	return assetWeight;
}

export function calculateMarketAvailablePNL(
	perpMarket: PerpMarketAccount,
	spotMarket: SpotMarketAccount
): BN {
	return getTokenAmount(
		perpMarket.pnlPool.scaledBalance,
		spotMarket,
		SpotBalanceType.DEPOSIT
	);
}

export function calculateMarketMaxAvailableInsurance(
	perpMarket: PerpMarketAccount,
	spotMarket: SpotMarketAccount
): BN {
	assert(spotMarket.marketIndex == QUOTE_SPOT_MARKET_INDEX);

	// todo: insuranceFundAllocation technically not guaranteed to be in Insurance Fund
	const insuranceFundAllocation =
		perpMarket.insuranceClaim.quoteMaxInsurance.sub(
			perpMarket.insuranceClaim.quoteSettledInsurance
		);
	const ammFeePool = getTokenAmount(
		perpMarket.amm.feePool.scaledBalance,
		spotMarket,
		SpotBalanceType.DEPOSIT
	);
	return insuranceFundAllocation.add(ammFeePool);
}

export function calculateNetUserPnl(
	perpMarket: PerpMarketAccount,
	oraclePriceData: OraclePriceData
): BN {
	const netUserPositionValue = perpMarket.amm.baseAssetAmountWithAmm
		.mul(oraclePriceData.price)
		.div(BASE_PRECISION)
		.div(PRICE_TO_QUOTE_PRECISION);

	const netUserCostBasis = perpMarket.amm.quoteAssetAmount;

	const netUserPnl = netUserPositionValue.add(netUserCostBasis);

	return netUserPnl;
}

export function calculateNetUserPnlImbalance(
	perpMarket: PerpMarketAccount,
	spotMarket: SpotMarketAccount,
	oraclePriceData: OraclePriceData
): BN {
	const netUserPnl = calculateNetUserPnl(perpMarket, oraclePriceData);

	const pnlPool = getTokenAmount(
		perpMarket.pnlPool.scaledBalance,
		spotMarket,
		SpotBalanceType.DEPOSIT
	);
	const feePool = getTokenAmount(
		perpMarket.amm.feePool.scaledBalance,
		spotMarket,
		SpotBalanceType.DEPOSIT
	);

	const imbalance = netUserPnl.sub(pnlPool.add(feePool));

	return imbalance;
}

export function calculateAvailablePerpLiquidity(
	market: PerpMarketAccount,
	oraclePriceData: OraclePriceData,
	dlob: DLOB,
	slot: number
): { bids: BN; asks: BN } {
	let [bids, asks] = calculateMarketOpenBidAsk(
		market.amm.baseAssetReserve,
		market.amm.minBaseAssetReserve,
		market.amm.maxBaseAssetReserve,
		market.amm.orderStepSize
	);

	asks = asks.abs();

	const bidPrice = calculateBidPrice(market, oraclePriceData);
	const askPrice = calculateAskPrice(market, oraclePriceData);

	for (const bid of dlob.getMakerLimitBids(
		market.marketIndex,
		slot,
		MarketType.PERP,
		oraclePriceData,
		askPrice
	)) {
		bids = bids.add(
			bid.order.baseAssetAmount.sub(bid.order.baseAssetAmountFilled)
		);
	}

	for (const ask of dlob.getMakerLimitAsks(
		market.marketIndex,
		slot,
		MarketType.PERP,
		oraclePriceData,
		bidPrice
	)) {
		asks = asks.add(
			ask.order.baseAssetAmount.sub(ask.order.baseAssetAmountFilled)
		);
	}

	return {
		bids: bids,
		asks: asks,
	};
}
