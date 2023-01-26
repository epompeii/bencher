use std::{collections::HashMap, str::FromStr};

use bencher_json::JsonMetric;
use literally::hmap;
use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{anychar, digit1, space1},
    combinator::{eof, map, map_res, peek},
    multi::{fold_many1, many_till},
    sequence::{delimited, tuple},
    IResult,
};
use ordered_float::OrderedFloat;

use crate::{
    results::{
        adapter_metrics::AdapterMetrics, adapter_results::AdapterResults, LATENCY_RESOURCE_ID,
    },
    Adapter, AdapterError, Settings,
};

pub struct AdapterRustCriterion;

impl Adapter for AdapterRustCriterion {
    fn parse(input: &str, settings: Settings) -> Result<AdapterResults, AdapterError> {
        let mut benchmark_metrics = Vec::new();

        let mut prior_line = None;
        for line in input.lines() {
            if let Ok((remainder, benchmark_metric)) = parse_criterion(prior_line, line) {
                if remainder.is_empty() {
                    benchmark_metrics.push(benchmark_metric);
                }
            }

            if let Ok((remainder, (thread, context, location))) = parse_panic(line) {
                if remainder.is_empty() {
                    if settings.allow_failure {
                        continue;
                    }

                    return Err(AdapterError::Panic {
                        thread,
                        context,
                        location,
                    });
                }
            }

            prior_line = Some(line);
        }

        Ok(benchmark_metrics
            .into_iter()
            .filter_map(|(benchmark_name, metric)| {
                Some((
                    benchmark_name.as_str().parse().ok()?,
                    AdapterMetrics {
                        inner: hmap! {
                            LATENCY_RESOURCE_ID.clone() => metric
                        },
                    },
                ))
            })
            .collect::<HashMap<_, _>>()
            .into())
    }
}

fn parse_criterion<'i>(
    prior_line: Option<&str>,
    input: &'i str,
) -> IResult<&'i str, (String, JsonMetric)> {
    map(
        many_till(anychar, parse_criterion_time),
        |(key_chars, metric)| {
            let mut key: String = key_chars.into_iter().collect();
            if key.is_empty() {
                key = prior_line.unwrap_or_default().into();
            }
            (key, metric)
        },
    )(input)
}

fn parse_criterion_time(input: &str) -> IResult<&str, JsonMetric> {
    map(
        tuple((
            tuple((space1, tag("time:"), space1)),
            parse_criterion_metric,
            eof,
        )),
        |(_, metric, _)| metric,
    )(input)
}

fn parse_criterion_metric(input: &str) -> IResult<&str, JsonMetric> {
    map(
        delimited(
            tag("["),
            tuple((
                parse_criterion_duration,
                space1,
                parse_criterion_duration,
                space1,
                parse_criterion_duration,
            )),
            tag("]"),
        ),
        |(lower_bound, _, value, _, upper_bound)| JsonMetric {
            value,
            lower_bound: Some(lower_bound),
            upper_bound: Some(upper_bound),
        },
    )(input)
}

#[allow(clippy::float_arithmetic)]
fn parse_criterion_duration(input: &str) -> IResult<&str, OrderedFloat<f64>> {
    map_res(
        tuple((parse_float, space1, parse_units)),
        |(duration, _, units)| -> Result<OrderedFloat<f64>, nom::Err<nom::error::Error<String>>> {
            Ok((to_f64(duration)? * units.as_nanos()).into())
        },
    )(input)
}

fn parse_panic(input: &str) -> IResult<&str, (String, String, String)> {
    map(
        tuple((
            tag("thread "),
            delimited(tag("'"), many_till(anychar, peek(tag("'"))), tag("'")),
            tag(" panicked at "),
            delimited(tag("'"), many_till(anychar, peek(tag("'"))), tag("'")),
            tag(", "),
            many_till(anychar, eof),
        )),
        |(_, (thread, _), _, (context, _), _, (location, _))| {
            (
                thread.into_iter().collect(),
                context.into_iter().collect(),
                location.into_iter().collect(),
            )
        },
    )(input)
}

pub enum Units {
    Pico,
    Nano,
    Micro,
    Milli,
    Sec,
}

fn parse_units(input: &str) -> IResult<&str, Units> {
    alt((
        map(tag("ps"), |_| Units::Pico),
        map(tag("ns"), |_| Units::Nano),
        map(tag("\u{3bc}s"), |_| Units::Micro),
        map(tag("\u{b5}s"), |_| Units::Micro),
        map(tag("ms"), |_| Units::Milli),
        map(tag("s"), |_| Units::Sec),
    ))(input)
}

impl Units {
    #[allow(clippy::float_arithmetic)]
    fn as_nanos(&self) -> f64 {
        match self {
            Self::Pico => 1.0 / 1_000.0,
            Self::Nano => 1.0,
            Self::Micro => 1_000.0,
            Self::Milli => 1_000_000.0,
            Self::Sec => 1_000_000_000.0,
        }
    }
}

fn parse_float(input: &str) -> IResult<&str, Vec<&str>> {
    fold_many1(
        alt((digit1, tag("."), tag(","))),
        Vec::new,
        |mut float_chars, float_char| {
            if float_char == "," {
                float_chars
            } else {
                float_chars.push(float_char);
                float_chars
            }
        },
    )(input)
}

fn to_f64(input: Vec<&str>) -> Result<f64, nom::Err<nom::error::Error<String>>> {
    let mut number = String::new();
    for floating_point in input {
        number.push_str(floating_point);
    }
    f64::from_str(&number)
        .map_err(|_e| nom::Err::Error(nom::error::make_error(number, nom::error::ErrorKind::Tag)))
}

#[cfg(test)]
pub(crate) mod test_rust {
    use bencher_json::JsonMetric;
    use pretty_assertions::assert_eq;

    use crate::{
        adapters::test_util::{convert_file_path, validate_metrics},
        Adapter, AdapterResults, Settings,
    };

    use super::{parse_criterion, parse_panic, AdapterRustCriterion};

    fn convert_rust_criterion(suffix: &str) -> AdapterResults {
        let file_path = format!("./tool_output/rust/criterion/{}.txt", suffix);
        convert_file_path::<AdapterRustCriterion>(&file_path, Settings::default())
    }

    #[test]
    fn test_parse_criterion() {
        for (index, (expected, input)) in [
            (
                Ok((
                    "",
                    (
                        "criterion_benchmark".into(),
                        JsonMetric {
                            value: 280.0.into(),
                            lower_bound: Some(222.2.into()),
                            upper_bound: Some(333.33.into()),
                        },
                    ),
                )),
                "criterion_benchmark                    time:   [222.2 ns 280.0 ns 333.33 ns]",
            ),
            (
                Ok((
                    "",
                    (
                        "criterion_benchmark".into(),
                        JsonMetric {
                            value: 5.280.into(),
                            lower_bound: Some(0.222.into()),
                            upper_bound: Some(0.33333.into()),
                        },
                    ),
                )),
                "criterion_benchmark                    time:   [222.0 ps 5,280.0 ps 333.33 ps]",
            ),
            (
                Ok((
                    "",
                    (
                        "criterion_benchmark".into(),
                        JsonMetric {
                            value: 18_019.0.into(),
                            lower_bound: Some(16_652.0.into()),
                            upper_bound: Some(19_562.0.into()),
                        },
                    ),
                )),
                "criterion_benchmark                    time:   [16.652 µs 18.019 µs 19.562 µs]",
            ),
        ]
        .into_iter()
        .enumerate()
        {
            assert_eq!(expected, parse_criterion(None, input), "#{index}: {input}")
        }
    }

    #[test]
    fn test_parse_panic() {
        for (index, (expected, input)) in [(
            Ok((
                "",
                (
                    "main".into(),
                    "explicit panic".into(),
                    "trace4rs/benches/trace4rs_bench.rs:42:5".into(),
                ),
            )),
            "thread 'main' panicked at 'explicit panic', trace4rs/benches/trace4rs_bench.rs:42:5",
        )]
        .into_iter()
        .enumerate()
        {
            assert_eq!(expected, parse_panic(input), "#{index}: {input}")
        }
    }

    #[test]
    fn test_adapter_rust_criterion() {
        let results = convert_rust_criterion("many");
        assert_eq!(results.inner.len(), 5);

        let metrics = results.get("file").unwrap();
        validate_metrics(metrics, 0.32389999999999997, Some(0.32062), Some(0.32755));

        let metrics = results.get("rolling_file").unwrap();
        validate_metrics(metrics, 0.42966000000000004, Some(0.38179), Some(0.48328));

        let metrics = results.get("tracing_file").unwrap();
        validate_metrics(metrics, 18019.0, Some(16652.0), Some(19562.0));

        let metrics = results.get("tracing_rolling_file").unwrap();
        validate_metrics(metrics, 20930.0, Some(18195.0), Some(24240.0));

        let metrics = results.get("benchmark: name with spaces").unwrap();
        validate_metrics(metrics, 20.930, Some(18.195), Some(24.240));
    }

    #[test]
    fn test_adapter_rust_criterion_failed() {
        let contents = std::fs::read_to_string("./tool_output/rust/criterion/failed.txt").unwrap();
        assert!(AdapterRustCriterion::parse(&contents, Settings::default()).is_err());
    }

    #[test]
    fn test_adapter_rust_criterion_failed_allow_failure() {
        let contents = std::fs::read_to_string("./tool_output/rust/criterion/failed.txt").unwrap();
        let results = AdapterRustCriterion::parse(
            &contents,
            Settings {
                allow_failure: true,
            },
        )
        .unwrap();
        assert_eq!(results.inner.len(), 4);
    }

    #[test]
    fn test_adapter_rust_criterion_dogfood() {
        let results = convert_rust_criterion("dogfood");
        assert_eq!(results.inner.len(), 4);

        let metrics = results.get("JsonAdapter::Magic (JSON)").unwrap();
        validate_metrics(
            metrics,
            3463.2000000000003,
            Some(3462.2999999999997),
            Some(3464.1000000000003),
        );

        let metrics = results.get("JsonAdapter::Json").unwrap();
        validate_metrics(metrics, 3479.6, Some(3479.2999999999997), Some(3480.0));

        let metrics = results.get("JsonAdapter::Magic (Rust)").unwrap();
        validate_metrics(metrics, 14726.0, Some(14721.0), Some(14730.0));

        let metrics = results.get("JsonAdapter::Rust").unwrap();
        validate_metrics(metrics, 14884.0, Some(14881.0), Some(14887.0));
    }
}
