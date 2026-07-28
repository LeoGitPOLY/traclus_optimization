pub mod gui_parallel_runner;
pub mod statistic;
pub mod data_type {
    pub mod angle_u16;
    pub mod point_i32_proj;
    pub mod decimal_i32;
}
pub mod events {
    pub mod app_events;
    pub mod event_singleton;
}
pub mod debug {
    pub mod logger;
    pub mod perf_timer;
}
