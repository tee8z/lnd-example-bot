use crate::{
    bot::Bot,
    configuration::{NodeSettings, Settings},
    lnd::LndManager,
};
use signal_hook::flag;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use tracing::instrument;

pub struct Application {
    bot: Bot,
}

impl Application {
    #[instrument(skip_all)]
    pub async fn build(configuration: Settings) -> Result<Self, anyhow::Error> {
        let bot = build_bot(configuration.lnd).await?;
        Ok(Self { bot })
    }
    #[instrument(skip_all)]
    pub async fn run_until_stopped(self) -> Result<(), std::io::Error> {
        flag::register(
            signal_hook::consts::SIGTERM,
            Arc::clone(&self.bot.kill_signal),
        )?;
        self.bot.await
    }
}

#[instrument(skip_all)]
pub async fn build_bot(node_configuration: NodeSettings) -> Result<Bot, anyhow::Error> {
    let kill_signal: Arc<AtomicBool> = Arc::new(AtomicBool::new(false));
    let lnd_manager = LndManager::build(
        node_configuration.url,
        node_configuration.pingfreqsecs,
        kill_signal.clone(),
        node_configuration.macaroonpath,
    );
    let bot = Bot {
        lnd_manager,
        kill_signal,
    };
    Ok(bot)
}
