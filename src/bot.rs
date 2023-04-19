use std::{
    io::ErrorKind,
    sync::{atomic::AtomicBool, Arc},
    task::{Context, Poll},
};

use crate::lnd::LndManager;
use futures_util::{future::join_all, Future};
use tracing::instrument;

#[derive(Clone)]
pub struct Bot {
    pub lnd_manager: LndManager,
    pub kill_signal: Arc<AtomicBool>,
}

impl Bot {
    #[instrument(skip_all)]
    pub async fn run(self) -> Result<(), std::io::Error> {
        //NOTE: add more managers (long living background processes) here
        let mut tasks = vec![];
        let node_manager_task = tokio::spawn(async { self.lnd_manager.await });
        tasks.push(node_manager_task);
        let all_tasks = join_all(tasks);
        all_tasks.await;
        Ok(())
    }
}

impl Future for Bot {
    type Output = Result<(), std::io::Error>;

    fn poll(self: std::pin::Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let async_fn = async { self.clone().run().await };

        //NOTE: Convert the async function to a future using `Box::pin`
        let mut future = Box::pin(async_fn);

        //NOTE: Poll the future using `poll` on the returned `Pin` reference
        match future.as_mut().poll(cx) {
            Poll::Ready(res) => match res {
                Ok(_) => Poll::Ready(Ok(())),
                Err(e) => Poll::Ready(Err(std::io::Error::new(
                    ErrorKind::Other,
                    format!("unexpected error in running bot tasks: {:?}", e),
                ))),
            },
            Poll::Pending => Poll::Pending,
        }
    }
}
