#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::core::{Error, Method, MovingAverageConstructor, PeriodType, Source, OHLCV};
use crate::core::{IndicatorConfig, IndicatorInstance, IndicatorResult};
use crate::helpers::MA;
use crate::methods::{Cross, RateOfChange, ReversalSignal};

/// Coppock curve
///
/// ## Links
///
/// * <https://en.wikipedia.org/wiki/Coppock_curve>
///
/// # 2 values
///
/// * `Main value`
///
/// Range of values is the same as the range of the `source` values.
///
/// * `Signal line` value
///
/// Range of values is the same as the range of the `source` values.
///
/// # 3 signals
///
/// * Signal 1 appears when `main value` crosses zero line. When `main value` crosses zero line upwards, returns full buy signal. When `main value` crosses zero line downwards, returns full sell signal.
/// * Signal 2 appears on reverse points of `main value`. When top reverse point appears,
/// * Signal 3 appears on `main value` crosses `signal line`. When `main value` crosses `signal line` upwards, returns full buy signal. When `main value` crosses `signal line` downwards, returns full sell signal.
#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct CoppockCurve<M: MovingAverageConstructor = MA> {
	/// Main MA type.
	///
	/// Default is [`WMA(10)`](crate::methods::WMA)
	///
	/// Period range in \[`2`; [`PeriodType::MAX`](crate::core::PeriodType)\).
	pub ma1: M,

	/// Signal line MA type .
	///
	/// Default is [`EMA(5)`](crate::methods::EMA)
	///
	/// Period range in \[`2`; [`PeriodType::MAX`](crate::core::PeriodType)\).
	pub s3_ma: M,

	/// Long rate of change period. Default is `14`.
	///
	/// Range in \(`period3`; [`PeriodType::MAX`](crate::core::PeriodType)\).
	pub period2: PeriodType,

	/// Short rate of change period. Default is `11`.
	///
	/// Range in \[`1`; `period2`\).
	pub period3: PeriodType,

	/// Signal 2 reverse points left limit. Default is `4`.
	///
	/// Range in \[`1`; [`PeriodType::MAX`](crate::core::PeriodType)-`s2_right`\).
	pub s2_left: PeriodType,

	/// Signal 2 reverse points right limit. Default is `2`
	///
	/// Range in \[`1`; [`PeriodType::MAX`](crate::core::PeriodType)-`s2_left`\).
	pub s2_right: PeriodType,

	/// Source type. Default is [`Close`](crate::core::Source::Close).
	pub source: Source,
}

impl<M: MovingAverageConstructor> IndicatorConfig for CoppockCurve<M> {
	type Instance = CoppockCurveInstance<M>;

	const NAME: &'static str = "CoppockCurve";

	fn init<T: OHLCV>(self, candle: &T) -> Result<Self::Instance, Error> {
		if !self.validate() {
			return Err(Error::WrongConfig);
		}

		let cfg = self;
		let src = &candle.source(cfg.source);
		Ok(Self::Instance {
			roc1: RateOfChange::new(cfg.period2, src)?,
			roc2: RateOfChange::new(cfg.period3, src)?,
			ma1: cfg.ma1.init(0.)?,   // method(cfg.method1, cfg.period1, 0.)?,
			ma2: cfg.s3_ma.init(0.)?, //method(cfg.method2, cfg.s3_period, 0.)?,
			cross_over1: Cross::default(),
			pivot: ReversalSignal::new(cfg.s2_left, cfg.s2_right, &0.)?,
			cross_over2: Cross::default(),

			cfg,
		})
	}

	fn validate(&self) -> bool {
		self.ma1.ma_period() > 1
			&& self.period2 > self.period3
			&& self.period2 < PeriodType::MAX
			&& self.period3 > 0
			&& self.s3_ma.ma_period() > 1
			&& self.s2_left > 0
			&& self.s2_right > 0
			&& self.s2_left.saturating_add(self.s2_right) < PeriodType::MAX
	}

	fn set(&mut self, name: &str, value: String) -> Result<(), Error> {
		match name {
			"ma1" => match value.parse() {
				Err(_) => return Err(Error::ParameterParse(name.to_string(), value.to_string())),
				Ok(value) => self.ma1 = value,
			},
			"period2" => match value.parse() {
				Err(_) => return Err(Error::ParameterParse(name.to_string(), value.to_string())),
				Ok(value) => self.period2 = value,
			},
			"period3" => match value.parse() {
				Err(_) => return Err(Error::ParameterParse(name.to_string(), value.to_string())),
				Ok(value) => self.period3 = value,
			},
			"s2_left" => match value.parse() {
				Err(_) => return Err(Error::ParameterParse(name.to_string(), value.to_string())),
				Ok(value) => self.s2_left = value,
			},
			"s2_right" => match value.parse() {
				Err(_) => return Err(Error::ParameterParse(name.to_string(), value.to_string())),
				Ok(value) => self.s2_right = value,
			},
			"s3_ma" => match value.parse() {
				Err(_) => return Err(Error::ParameterParse(name.to_string(), value.to_string())),
				Ok(value) => self.s3_ma = value,
			},
			"source" => match value.parse() {
				Err(_) => return Err(Error::ParameterParse(name.to_string(), value.to_string())),
				Ok(value) => self.source = value,
			},
			// "zone"		=> self.zone = value.parse().unwrap(),
			// "source"	=> self.source = value.parse().unwrap(),
			_ => {
				return Err(Error::ParameterParse(name.to_string(), value));
			}
		};

		Ok(())
	}

	fn size(&self) -> (u8, u8) {
		(2, 3)
	}
}

impl Default for CoppockCurve<MA> {
	fn default() -> Self {
		Self {
			ma1: MA::WMA(10),
			s3_ma: MA::EMA(5),
			period2: 14,
			period3: 11,
			s2_left: 4,
			s2_right: 2,
			source: Source::Close,
		}
	}
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct CoppockCurveInstance<M: MovingAverageConstructor = MA> {
	cfg: CoppockCurve<M>,

	roc1: RateOfChange,
	roc2: RateOfChange,
	ma1: M::Instance,
	ma2: M::Instance,
	cross_over1: Cross,
	pivot: ReversalSignal,
	cross_over2: Cross,
}

impl<M: MovingAverageConstructor> IndicatorInstance for CoppockCurveInstance<M> {
	type Config = CoppockCurve<M>;

	fn config(&self) -> &Self::Config {
		&self.cfg
	}

	fn next<T: OHLCV>(&mut self, candle: &T) -> IndicatorResult {
		let src = &candle.source(self.cfg.source);
		let roc1 = self.roc1.next(src);
		let roc2 = self.roc2.next(src);
		let value1 = self.ma1.next(&(roc1 + roc2));
		let value2 = self.ma2.next(&value1);

		let signal1 = self.cross_over1.next(&(value1, 0.));
		let signal2 = self.pivot.next(&value1);
		let signal3 = self.cross_over2.next(&(value1, value2));

		IndicatorResult::new(&[value1, value2], &[signal1, signal2, signal3])
	}
}
