use std::fs::OpenOptions;

use dognut::department::{
    common::constant::{self},
    pipeline::{rasterizer::RasterRunner, shader::LambertianShader},
    preview::vector::Vector3,
    tui::TuiApp,
    types::{msg, multi_sender::MultiSender},
};
use log::LevelFilter;

use dognut::{department::view::camera::Camera, util::ARG};

use log::{error, info};

fn main() {
    let log_file = OpenOptions::new()
        .create(true)
        .append(true)
        .open("ctree.log")
        .expect("无法创建日志文件");

    let env = env_logger::Env::default();
    env_logger::Builder::from_env(env)
        // 将 Target::Stdout 改为 Target::File
        .target(env_logger::Target::Pipe(Box::new(log_file)))
        //.target(env_logger::Target::Stdout)
        .filter(Some("wgpu_core"), LevelFilter::Off)
        .filter_level(LevelFilter::Error)
        .format_timestamp_millis()
        .init();

    let arg = &ARG;

    info!(target:"wgpu_core", "hello");

    let (net_sender, net_receiver) = crossbeam_channel::unbounded::<msg::TransferMsg>();
    let (win_sender, win_receiver) = crossbeam_channel::unbounded::<msg::TransferMsg>();
    let (enc_sender, enc_receiver) = crossbeam_channel::unbounded::<msg::TransferMsg>();

    let ms = MultiSender::new(net_sender, enc_sender, win_sender);

    let camera = Camera::new(
        45.,
        (constant::WIDTH / constant::HEIGHT) as f32,
        -5.,
        -50.,
        Vector3::from_xyz(0., 0., 10.),
        Vector3::from_xyz(0., 0., -1.),
        Vector3::from_xyz(0., -1., 0.),
    );

    let shader = LambertianShader::new(Vector3::from_xyz(0., 1., 0.), 0.8, 1., &camera, arg.term);

    let raster = RasterRunner::new(ms.clone(), camera, Box::new(shader), arg.term);


    let inner_rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap();

    inner_rt.block_on(async move {
        let result = TuiApp::new(raster).run(None).await;
        if let Err(e) = result {
            error!("tui return an error, {}", e.to_string());
        };
    });

    info!("tui app end");
}
