use crate::config::Config;
use crate::model::{License, MoneyList, Tile};
use async_std::channel::{bounded, unbounded};
use async_std::channel::{Receiver, Sender};
use core::num::NonZeroU32;
use governor::clock::DefaultClock;
use governor::state::{InMemoryState, NotKeyed};
use governor::{Quota, RateLimiter};
use num_integer::Integer;
use std::convert::TryFrom;
use std::sync::{Arc, Mutex, RwLock};

#[derive(Clone)]
pub struct SyncContext {
    pub area_sender: Sender<Tile>,
    pub area_receiver: Receiver<Tile>,
    pub tile_sender: Sender<Tile>,
    pub tile_receiver: Receiver<Tile>,
    pub license_sender: Sender<License>,
    pub license_receiver: Receiver<License>,
    pub empty_license_sender: Sender<License>,
    pub empty_license_receiver: Receiver<License>,
    pub treasure_sender: Sender<String>,
    pub treasure_receiver: Receiver<String>,
    pub cash_sender: Sender<MoneyList>,
    pub cash_receiver: Receiver<MoneyList>,
    pub metrics_sender: Sender<Metrics>,
    pub metrics_receiver: Receiver<Metrics>,
    pub metrics: Arc<Mutex<Metrics>>,
    http_rate_limiter: Arc<RateLimiter<NotKeyed, InMemoryState, DefaultClock>>,
    explore_phase1_rate_limiter: Arc<RateLimiter<NotKeyed, InMemoryState, DefaultClock>>,
    explore_phase2_rate_limiter: Arc<RateLimiter<NotKeyed, InMemoryState, DefaultClock>>,
    digger_phase1_rate_limiter: Arc<RateLimiter<NotKeyed, InMemoryState, DefaultClock>>,
    digger_phase2_rate_limiter: Arc<RateLimiter<NotKeyed, InMemoryState, DefaultClock>>,
    attorney_phase1_rate_limiter: Arc<RateLimiter<NotKeyed, InMemoryState, DefaultClock>>,
    attorney_phase2_rate_limiter: Arc<RateLimiter<NotKeyed, InMemoryState, DefaultClock>>,
    accountant_phase1_rate_limiter: Arc<RateLimiter<NotKeyed, InMemoryState, DefaultClock>>,
    accountant_phase2_rate_limiter: Arc<RateLimiter<NotKeyed, InMemoryState, DefaultClock>>,
    phase2: Arc<RwLock<bool>>,
}

impl SyncContext {
    pub fn new(c: Config) -> SyncContext {
        let (area_sender, area_receiver) = bounded(c.area_chan_cap);
        let (tile_sender, tile_receiver) = bounded(c.tile_chan_cap);
        let (license_sender, license_receiver) = bounded(c.license_chan_cap);
        let (empty_license_sender, empty_license_receiver) = bounded(c.empty_license_chan_cap);
        let (treasure_sender, treasure_receiver) = bounded(c.treasure_chan_cap);
        let (cash_sender, cash_receiver) = unbounded();
        let (metrics_sender, metrics_receiver) = unbounded();

        SyncContext {
            area_sender,
            area_receiver,
            tile_sender,
            tile_receiver,
            license_sender,
            license_receiver,
            empty_license_sender,
            empty_license_receiver,
            treasure_sender,
            treasure_receiver,
            cash_sender,
            cash_receiver,
            metrics_sender,
            metrics_receiver,
            metrics: Arc::new(Mutex::new(Metrics::new())),
            http_rate_limiter: Arc::new(RateLimiter::direct(Quota::per_second(
                NonZeroU32::try_from(c.max_rps).unwrap(),
            ))),
            explore_phase1_rate_limiter: Arc::new(RateLimiter::direct(Quota::per_second(
                NonZeroU32::try_from(c.explore_phase1_rps).unwrap(),
            ))),
            explore_phase2_rate_limiter: Arc::new(RateLimiter::direct(Quota::per_second(
                NonZeroU32::try_from(c.explore_phase2_rps).unwrap(),
            ))),
            digger_phase1_rate_limiter: Arc::new(RateLimiter::direct(Quota::per_second(
                NonZeroU32::try_from(c.digger_phase1_rps).unwrap(),
            ))),
            digger_phase2_rate_limiter: Arc::new(RateLimiter::direct(Quota::per_second(
                NonZeroU32::try_from(c.digger_phase2_rps).unwrap(),
            ))),
            attorney_phase1_rate_limiter: Arc::new(RateLimiter::direct(Quota::per_second(
                NonZeroU32::try_from(c.attorney_phase1_rps).unwrap(),
            ))),
            attorney_phase2_rate_limiter: Arc::new(RateLimiter::direct(Quota::per_second(
                NonZeroU32::try_from(c.attorney_phase2_rps).unwrap(),
            ))),
            accountant_phase1_rate_limiter: Arc::new(RateLimiter::direct(Quota::per_second(
                NonZeroU32::try_from(c.accountant_phase1_rps).unwrap(),
            ))),
            accountant_phase2_rate_limiter: Arc::new(RateLimiter::direct(Quota::per_second(
                NonZeroU32::try_from(c.accountant_phase2_rps).unwrap(),
            ))),
            phase2: Arc::new(RwLock::new(false)),
        }
    }
    pub async fn init(&self) {
        for _ in 0..self.empty_license_sender.capacity().unwrap() {
            &self.empty_license_sender.send(License::new()).await;
        }
    }
    pub async fn http_rate_limiter(&self) {
        self.http_rate_limiter.until_ready().await
    }

    pub async fn explore_rate_limiter(&self) {
        if *self.phase2.read().unwrap() {
            self.explore_phase2_rate_limiter.until_ready().await
        } else {
            self.explore_phase1_rate_limiter.until_ready().await
        }
    }
    pub async fn digger_rate_limiter(&self) {
        if *self.phase2.read().unwrap() {
            self.digger_phase2_rate_limiter.until_ready().await
        } else {
            self.digger_phase1_rate_limiter.until_ready().await
        }
    }
    pub async fn attorney_rate_limiter(&self) {
        if *self.phase2.read().unwrap() {
            self.attorney_phase2_rate_limiter.until_ready().await
        } else {
            self.attorney_phase1_rate_limiter.until_ready().await
        }
    }
    pub async fn accountant_rate_limiter(&self) {
        if *self.phase2.read().unwrap(){
            self.accountant_phase2_rate_limiter.until_ready().await
        } else {
            self.accountant_phase1_rate_limiter.until_ready().await
        }
    }
    pub fn activate_phase2(&mut self) {
        let mut phase2 = self.phase2.write().unwrap();
        *phase2 = true
    }
    pub fn activate_phase1(&mut self) {
        let mut phase2 = self.phase2.write().unwrap();
        *phase2 = false
    }
    pub fn switch_phase(&mut self) {
        if *self.phase2.read().unwrap() {
            self.activate_phase1()
        } else {
            self.activate_phase2()
        }
    }
    pub fn is_phase2(&self) -> bool{
        *self.phase2.read().unwrap()
    }
}

#[derive(Copy, Clone)]
pub struct Metrics {
    pub dig_count: u64,
    pub dig_success: u64,
    pub dig_price: f32,
    pub cash_count: u64,
    pub cash_success: u64,
    pub cash_value: u64,
    pub cash_price: f32,
    pub license_count: u64,
    pub license_value: u64,
    pub license_price: u64,
    pub explore_count: u64,
    pub explore_success: u64,
    pub explore_price: f32,
    pub explore_odd_x: u64,
    pub explore_odd_y: u64,
    pub http200: u64,
    pub http404: u64,
    pub http409: u64,
    pub http422: u64,
    pub http429: u64,
    pub http50x: u64,
    pub http_other: u64,
}

impl Metrics {
    pub fn new() -> Metrics {
        Metrics {
            dig_count: 0,
            dig_success: 0,
            dig_price: 0.0,
            cash_count: 0,
            cash_success: 0,
            cash_value: 0,
            cash_price: 0.0,
            license_count: 0,
            license_value: 0,
            license_price: 0,
            explore_count: 0,
            explore_success: 0,
            explore_price: 0.0,
            explore_odd_x: 0,
            explore_odd_y: 0,
            http200: 0,
            http404: 0,
            http409: 0,
            http422: 0,
            http429: 0,
            http50x: 0,
            http_other: 0,
        }
    }
    pub fn new200() -> Metrics {
        let mut m = Metrics::new();
        m.http200 += 1;
        m
    }
    pub fn new404() -> Metrics {
        let mut m = Metrics::new();
        m.http404 += 1;
        m
    }
    pub fn new409() -> Metrics {
        let mut m = Metrics::new();
        m.http409 += 1;
        m
    }
    pub fn new422() -> Metrics {
        let mut m = Metrics::new();
        m.http422 += 1;
        m
    }
    pub fn new429() -> Metrics {
        let mut m = Metrics::new();
        m.http429 += 1;
        m
    }
    pub fn new50x() -> Metrics {
        let mut m = Metrics::new();
        m.http50x += 1;
        m
    }
    pub fn new_other() -> Metrics {
        let mut m = Metrics::new();
        m.http_other += 1;
        m
    }
    pub fn new_cash(value: u64, success: bool) -> Metrics {
        let mut m = Metrics::new();
        m.cash_count += 1;
        m.cash_price = 10.0;
        if success {
            m.cash_success += 1;
            m.cash_value += value;
        }
        m
    }
    pub fn new_license(value: u64, price: u64) -> Metrics {
        let mut m = Metrics::new();
        m.license_count += 1;
        m.license_value += value;
        m.license_price += price;
        m
    }
    pub fn new_dig(success: bool, _depth: u64) -> Metrics {
        let mut m = Metrics::new();
        m.dig_count += 1;
        m.dig_price = ( 2.1 + 0.18*(_depth as f32 - 1.0) )/ 2.0;
        if success {
            m.dig_success += 1;
        }
        m
    }
    pub fn new_explore(success: bool, x: u64, y: u64, size_x: u64, size_y: u64) -> Metrics {
        let mut m = Metrics::new();
        m.explore_count += 1;
        if success {
            m.explore_success += 1;
        }
        if success && x.is_odd() {
            m.explore_odd_x += 1;
        }
        if success && y.is_odd() {
            m.explore_odd_y += 1;
        }

        let square = size_x * size_y;
        m.explore_price = if square == 0 {
            0.0
        } else if square < 4 {
            0.5
        } else if square < 8 {
            1.0
        } else if square < 16 {
            1.5
        } else if square < 32 {
            2.0
        } else if square < 64 {
            2.5
        } else if square < 128 {
            3.0
        } else if square < 256 {
            3.5
        } else if square < 512{
            4.0
        } else if square < 1024{
            4.5
        } else {
            100.0
        };
        m
    }
    pub fn new_calculated_explore(success: bool, x: u64, y: u64) -> Metrics {
        let mut m = Metrics::new();
        if success {
            m.explore_success += 1;
        }
        if success && x.is_odd() {
            m.explore_odd_x += 1;
        }
        if success && y.is_odd() {
            m.explore_odd_y += 1;
        }
        m
    }
    pub fn sum_http(&self) -> u64 {
        self.http200
            + self.http404
            + self.http409
            + self.http422
            + self.http429
            + self.http50x
            + self.http_other
    }
    pub fn sum_price(&self) -> f32{
        self.dig_price + self.cash_price + self.explore_price
    }
    pub fn rps_http(&self, old_metrics: Metrics, period_sec: u64) -> u64 {
        (self.sum_http() - old_metrics.sum_http()) / period_sec
    }
    pub fn rps_price(&self, old_metrics: Metrics, period_sec: u64) -> f32 {
        (self.sum_price() - old_metrics.sum_price()) / period_sec as f32
    }
    pub fn rps_price_explore(&self, old_metrics: Metrics, period_sec: u64) -> f32 {
        (self.explore_price - old_metrics.explore_price) / period_sec as f32
    }
    pub fn rps_price_dig(&self, old_metrics: Metrics, period_sec: u64) -> f32 {
        (self.dig_price - old_metrics.dig_price) / period_sec as f32
    }
    pub fn rps_price_cash(&self, old_metrics: Metrics, period_sec: u64) -> f32 {
        (self.cash_price - old_metrics.cash_price) / period_sec as f32
    }
    pub fn rps_explore(&self, old_metrics: Metrics, period_sec: u64) -> u64 {
        (self.explore_count - old_metrics.explore_count) / period_sec
    }
    pub fn rps_license(&self, old_metrics: Metrics, period_sec: u64) -> u64 {
        (self.license_count - old_metrics.license_count) / period_sec
    }
    pub fn rps_cash(&self, old_metrics: Metrics, period_sec: u64) -> u64 {
        (self.cash_count - old_metrics.cash_count) / period_sec
    }
    pub fn rps_cash_success(&self, old_metrics: Metrics, period_sec: u64) -> u64 {
        (self.cash_success - old_metrics.cash_success) / period_sec
    }
    pub fn rps_dig(&self, old_metrics: Metrics, period_sec: u64) -> u64 {
        (self.dig_count - old_metrics.dig_count) / period_sec
    }
    pub fn add(&mut self, other: Metrics) {
        self.dig_count += other.dig_count;
        self.dig_success += other.dig_success;
        self.dig_price += other.dig_price;
        self.explore_odd_x += other.explore_odd_x;
        self.explore_odd_y += other.explore_odd_y;
        self.explore_price += other.explore_price;
        self.cash_count += other.cash_count;
        self.cash_success += other.cash_success;
        self.cash_value += other.cash_value;
        self.cash_price += other.cash_price;
        self.license_count += other.license_count;
        self.license_value += other.license_value;
        self.license_price += other.license_price;
        self.explore_count += other.explore_count;
        self.explore_success += other.explore_success;
        self.http200 += other.http200;
        self.http404 += other.http404;
        self.http409 += other.http409;
        self.http422 += other.http422;
        self.http429 += other.http429;
        self.http50x += other.http50x;
        self.http_other += other.http_other;
    }
}
