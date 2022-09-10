use std::convert::TryFrom;

use async_trait::async_trait;
use bencher_json::ResourceId;
use uuid::Uuid;

use crate::{
    bencher::{backend::Backend, sub::SubCmd, wide::Wide},
    cli::report::CliReportView,
    CliError,
};

#[derive(Debug)]
pub struct View {
    pub project: ResourceId,
    pub report: Uuid,
    pub backend: Backend,
}

impl TryFrom<CliReportView> for View {
    type Error = CliError;

    fn try_from(view: CliReportView) -> Result<Self, Self::Error> {
        let CliReportView {
            project,
            report,
            backend,
        } = view;
        Ok(Self {
            project,
            report,
            backend: backend.try_into()?,
        })
    }
}

#[async_trait]
impl SubCmd for View {
    async fn exec(&self, _wide: &Wide) -> Result<(), CliError> {
        self.backend
            .get(&format!(
                "/v0/projects/{}/reports/{}",
                self.project, self.report
            ))
            .await?;
        Ok(())
    }
}
