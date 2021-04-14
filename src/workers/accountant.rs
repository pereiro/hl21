use crate::context::{Metrics, SyncContext};
use crate::http::http_post;
use crate::model::MoneyList;
use std::time::Duration;
use url::Url;

pub struct Accountant {
    url: Url,
    timeout: Duration,
    client: surf::Client,
    sync: SyncContext,
}

impl Accountant {
    pub fn new(url: Url, timeout: Duration, sync: SyncContext) -> Accountant {
        Accountant {
            url,
            timeout,
            client: surf::Client::new(),
            sync,
        }
    }
    pub async fn start(self) {
        loop {
            let treasure = self.sync.treasure_receiver.recv().await.unwrap();
            self.sync.accountant_rate_limiter().await;
            let money: MoneyList = match http_post(
                &self.url,
                self.timeout,
                treasure.clone(),
                &self.client,
                self.sync.clone(),
            )
            .await
            {
                Ok(m) => m,
                Err(_) => {
                    self.sync.treasure_sender.send(treasure).await.unwrap();

                    self.sync
                        .metrics_sender
                        .send(Metrics::new_cash(0u64, false))
                        .await
                        .unwrap();
                    continue;
                }
            };

            self.sync
                .metrics_sender
                .send(Metrics::new_cash(money.len() as u64, true))
                .await
                .unwrap();


            self.sync.cash_sender.send(money).await.unwrap();
        }
    }
}
