use jiff::{
    Span, SpanRound, Unit,
    civil::{Date, DateTime},
    fmt::friendly::{Designator, Direction, FractionalUnit, Spacing, SpanPrinter},
};
use serde::Deserialize;
use tsify::Tsify;
use wasm_bindgen::prelude::*;

/// Get a friendly description of the amount of time between two dates
#[wasm_bindgen]
pub fn diff(
    start: &str,
    start_time_zone: &str,
    end: &str,
    end_time_zone: &str,
    options: Options,
) -> Result<String, JsError> {
    let options = PrinterOptions::try_from(options)?;
    let start_date = start.parse::<DateTime>()?.in_tz(start_time_zone)?;
    let end_date = end.parse::<DateTime>()?.in_tz(end_time_zone)?;
    let diff = start_date.until((Unit::Year, &end_date))?;
    let printer = options.into_printer();
    let friendly_diff = printer.span_to_string(&diff);
    Ok(friendly_diff)
}

/// Get a friendly description of this duration, optionally using a relative date
#[wasm_bindgen]
pub fn duration(
    duration: &str,
    relative_date: Option<String>,
    options: Options,
) -> Result<String, JsError> {
    let duration: Span = duration.parse()?;
    let options = PrinterOptions::try_from(options)?;
    let printer = options.into_printer();
    let output = if let Some(relative) = relative_date.map(|x| x.parse::<Date>()).transpose()? {
        let rounding_options = SpanRound::new().relative(relative).largest(Unit::Year);
        printer.span_to_string(&duration.round(rounding_options)?)
    } else {
        printer.span_to_string(&duration)
    };
    Ok(output)
}

/// Get a list of all time zones
#[wasm_bindgen]
#[must_use]
pub fn list_time_zones() -> Vec<String> {
    jiff::tz::db().available().map(|x| x.to_string()).collect()
}

/// Customize the output
///
/// See [`SpanPrinter`]
#[derive(Tsify, Deserialize)]
#[tsify(from_wasm_abi)]
pub struct Options {
    /// How units and designators are spaced
    ///
    /// See [`Spacing`]
    #[tsify(type = r#""none" | "between-units-and-designators" | "between-units""#)]
    pub spacing: String,

    /// Whether there should be a comma after the designator
    pub comma_after_designator: bool,

    /// The format of the designator
    ///
    /// See [`Designator`]
    #[tsify(type = r#""short" | "compact" | "verbose" | "human-time""#)]
    pub designator: String,

    /// If hours, minutes, and seconds should be formatted as HH:MM:SS
    ///
    pub hours_minutes_seconds: bool,

    /// The precision to use for fractional times
    ///
    /// See [`FractionalUnit`]
    #[tsify(
        optional,
        type = r#""hour" | "minute" | "second" | "millisecond" | "microsecond""#
    )]
    pub fractional_unit: Option<String>,

    /// The padding to use when writing unit values
    pub padding: u8,

    /// The unit to use when the duration is zero
    ///
    /// See [`Unit`]
    #[tsify(
        type = r#""year" | "month" | "week" | "day" | "hour" | "minute" | "second" | "millisecond" | "microsecond" | "nanosecond""#
    )]
    pub zero_unit: String,

    /// How and when the sign for the duration is written
    ///
    /// See [`Direction`]
    #[tsify(type = r#""auto" | "sign" | "suffix" | "force-sign""#)]
    pub direction: String,
}

struct PrinterOptions(SpanPrinter);

impl PrinterOptions {
    fn into_printer(self) -> SpanPrinter {
        self.0
    }
}

impl TryFrom<Options> for PrinterOptions {
    type Error = JsError;

    fn try_from(value: Options) -> Result<Self, Self::Error> {
        let spacing = match value.spacing.as_str() {
            "none" => Ok(Spacing::None),
            "between-units-and-designators" => Ok(Spacing::BetweenUnitsAndDesignators),
            "between-units" => Ok(Spacing::BetweenUnits),
            x => Err(JsError::new(&format!("Invalid spacing option: {x:?}"))),
        }?;

        let designator = match value.designator.as_str() {
            "short" => Ok(Designator::Short),
            "compact" => Ok(Designator::Compact),
            "verbose" => Ok(Designator::Verbose),
            "human-time" => Ok(Designator::HumanTime),
            x => Err(JsError::new(&format!("Invalid designator option: {x:?}"))),
        }?;

        let fractional_unit = match value.fractional_unit.as_deref() {
            None => Ok(None),
            Some("hour") => Ok(Some(FractionalUnit::Hour)),
            Some("minute") => Ok(Some(FractionalUnit::Minute)),
            Some("second") => Ok(Some(FractionalUnit::Second)),
            Some("millisecond") => Ok(Some(FractionalUnit::Millisecond)),
            Some("microsecond") => Ok(Some(FractionalUnit::Microsecond)),
            Some(x) => Err(JsError::new(&format!("Invalid fractional unit: {x:?}"))),
        }?;

        let zero_unit = match value.zero_unit.as_str() {
            "year" => Ok(Unit::Year),
            "month" => Ok(Unit::Month),
            "week" => Ok(Unit::Week),
            "day" => Ok(Unit::Day),
            "hour" => Ok(Unit::Hour),
            "minute" => Ok(Unit::Minute),
            "second" => Ok(Unit::Second),
            "millisecond" => Ok(Unit::Millisecond),
            "microsecond" => Ok(Unit::Microsecond),
            "nanosecond" => Ok(Unit::Nanosecond),
            x => Err(JsError::new(&format!("Invalid zero unit: {x:?}"))),
        }?;

        let direction = match value.direction.as_str() {
            "auto" => Ok(Direction::Auto),
            "sign" => Ok(Direction::Sign),
            "suffix" => Ok(Direction::Suffix),
            "force-sign" => Ok(Direction::ForceSign),
            x => Err(JsError::new(&format!("Invalid direction: {x:?}"))),
        }?;

        let printer = SpanPrinter::new()
            .spacing(spacing)
            .comma_after_designator(value.comma_after_designator)
            .designator(designator)
            .hours_minutes_seconds(value.hours_minutes_seconds)
            .padding(value.padding)
            .zero_unit(zero_unit)
            .direction(direction)
            .fractional(fractional_unit);

        Ok(Self(printer))
    }
}
