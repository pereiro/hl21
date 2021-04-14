use crate::context::{Metrics, SyncContext};
use async_std::task;
use std::time::{Duration, Instant};

pub struct Statist {
    display_period_sec: u64,
    sync: SyncContext,
}

impl Statist {
    pub fn new(display_period_sec: u64, sync: SyncContext) -> Statist {
        Statist {
            display_period_sec,
            sync,
        }
    }
    pub async fn start(self) {
        let start = Instant::now();
        let http_metrics = self.sync.metrics.clone();
        let http_metrics_receiver = self.sync.metrics_receiver.clone();
        task::spawn(async move {
            loop {
                let m = http_metrics_receiver.recv().await.unwrap();
                drop(http_metrics.lock().unwrap().add(m));
            }
        });
        let mut old_metrics = Metrics::new();
        loop {
            let hm: Metrics = *self.sync.metrics.lock().unwrap();
            println!(
                "{}({}): a={},tl={},l={},tr={}|409={},422={},429={},5x={},Х={}|r:{}|e={}|l={}|d={}|c={}|cs={}|er={:.3}|pc={:.3}|pps={:.3}|tf={},tc={}({}),avg_t={:.3}|price:[pps:{:.0}({:.0}+{:.0}+{:.0}),e={:.0},d={:.0},c={:.0}],{}-{}={}",
                start.elapsed().as_secs(),
                self.sync.is_phase2(),
                self.sync.area_receiver.len(),
                self.sync.tile_receiver.len(),
                self.sync.license_receiver.len(),
                self.sync.treasure_receiver.len(),
                hm.http409,
                hm.http422,
                hm.http429,
                hm.http50x,
                hm.http_other,
                hm.rps_http(old_metrics, self.display_period_sec),
                hm.rps_explore(old_metrics, self.display_period_sec),
                hm.rps_license(old_metrics, self.display_period_sec),
                hm.rps_dig(old_metrics, self.display_period_sec),
                hm.rps_cash(old_metrics, self.display_period_sec),
                hm.rps_cash_success(old_metrics, self.display_period_sec),
                match hm.explore_count {
                    0 => 0.0,
                    _ => hm.explore_success as f32/hm.explore_count as f32
                },
                match hm.explore_count {
                    0 => 0.0,
                    _ => hm.explore_price as f32/hm.explore_count as f32
                },
                match hm.explore_count {
                    0 => 0.0,
                    _ => hm.explore_price as f32/hm.explore_success as f32
                },
                hm.dig_success,
                hm.cash_success,
                hm.cash_count,
                match hm.cash_success {
                    0 => 0.0,
                    _ => hm.cash_value as f32/hm.cash_success as f32
                },
                hm.rps_price(old_metrics,self.display_period_sec),
                hm.rps_price_explore(old_metrics,self.display_period_sec),
                hm.rps_price_dig(old_metrics,self.display_period_sec),
                hm.rps_price_cash(old_metrics,self.display_period_sec),
                hm.explore_price,
                hm.dig_price,
                hm.cash_price,
                hm.cash_value,
                hm.license_price,
                 hm.cash_value - hm.license_price,
            );
            old_metrics = hm;

            task::sleep(Duration::from_secs(self.display_period_sec)).await;
        }
    }
}

