use std::{collections::HashMap, str::FromStr};

use bencher_json::JsonMetric;
use serde::{Deserialize, Serialize};

use super::{CombinedKind, MetricKind, OrdKind};

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct AdapterMetrics {
    #[serde(flatten)]
    pub inner: MetricsMap,
}

pub type MetricsMap = HashMap<MetricKind, JsonMetric>;

impl From<MetricsMap> for AdapterMetrics {
    fn from(inner: MetricsMap) -> Self {
        Self { inner }
    }
}

impl AdapterMetrics {
    #[allow(clippy::arithmetic_side_effects)]
    pub(crate) fn combined(self, mut other: Self, kind: CombinedKind) -> Self {
        let mut metric_map = HashMap::new();
        for (metric_kind, metric) in self.inner {
            let other_metric = other.inner.remove(&metric_kind);
            let combined_metric = if let Some(other_metric) = other_metric {
                match kind {
                    CombinedKind::Ord(ord_kind) => match ord_kind {
                        OrdKind::Min => metric.min(other_metric),
                        OrdKind::Max => metric.max(other_metric),
                    },
                    CombinedKind::Add => metric + other_metric,
                }
            } else {
                metric
            };
            metric_map.insert(metric_kind, combined_metric);
        }
        metric_map.extend(other.inner);
        metric_map.into()
    }

    pub fn get(&self, key: &str) -> Option<&JsonMetric> {
        self.inner.get(&MetricKind::from_str(key).ok()?)
    }
}

impl std::ops::Div<usize> for AdapterMetrics {
    type Output = Self;

    #[allow(clippy::arithmetic_side_effects)]
    fn div(self, rhs: usize) -> Self::Output {
        let mut metric_map = HashMap::new();
        for (metric_kind, metric) in self.inner {
            metric_map.insert(metric_kind, metric / rhs);
        }
        metric_map.into()
    }
}
