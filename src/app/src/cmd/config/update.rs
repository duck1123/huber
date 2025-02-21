use std::io::stdout;

use async_trait::async_trait;
use clap::{App, ArgMatches};

use huber_common::di::DIContainer;
use huber_common::model::config::Config;
use huber_common::output::factory::FactoryConsole;
use huber_common::output::OutputTrait;
use huber_common::result::Result;
use huber_procmacro::process_lock;

use crate::cmd::{process_arg_matches, CommandAsyncTrait, CommandTrait};
use crate::service::config::{ConfigService, ConfigTrait};
use huber_common::model::config::ConfigPath;

pub(crate) const CMD_NAME: &str = "update";

#[derive(Debug)]
pub(crate) struct ConfigUpdateCmd;

unsafe impl Send for ConfigUpdateCmd {}

unsafe impl Sync for ConfigUpdateCmd {}

impl ConfigUpdateCmd {
    pub(crate) fn new() -> Self {
        Self {}
    }
}

impl<'a, 'b> CommandTrait<'a, 'b> for ConfigUpdateCmd {
    fn app(&self) -> App<'a, 'b> {
        App::new(CMD_NAME)
            .visible_alias("a")
            .about("Updates the configuration")
    }
}

#[async_trait]
impl<'a, 'b> CommandAsyncTrait<'a, 'b> for ConfigUpdateCmd {
    async fn run(
        &self,
        _config: &Config,
        container: &DIContainer,
        matches: &ArgMatches<'a>,
    ) -> Result<()> {
        process_lock!();

        let config_service = container.get::<ConfigService>().unwrap();

        println!("Updating the configuration");

        let mut c = config_service.get()?;
        if process_arg_matches(&mut c, &matches) {
            config_service.update(&c)?;
            println!("The configuration updated\n");
        } else {
            println!("Nothing changed. Please specify options for updating the configuration\n")
        }

        output!(c.output_format, .display(
            stdout(),
            &c,
            None,
            Some(vec!["home_dir"]),
        ))
    }
}
