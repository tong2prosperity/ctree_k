use std::fs::OpenOptions;

use dognut::department::{
    common::{
        constant::{self, HEIGHT, WIDTH},
        self_type,
    },
    pipeline::{rasterizer::RasterRunner, shader::LambertianShader},
    preview::vector::Vector3,
    tui::TuiApp,
    types::{msg, multi_sender::MultiSender},
};

use dognut::department::view::window;
use log::LevelFilter;

use dognut::{department::view::camera::Camera, util::ARG};

use log::{debug, error, info, warn};

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

    //router::Router::new(net_receiver, ms.clone()).run();
    //#[cfg(feature = "rtc")]
    //RgbaEncoder::run(enc_receiver, ms.clone(), (WIDTH, HEIGHT));

    //#[cfg(feature = "image_encoder")]
    //ImgEncoder::run(enc_receiver, ms.clone(), ( if arg.term { 256} else {WIDTH}, if arg.term {79} else {HEIGHT}));

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

    let res = dognut::department::model::object_loader::ObjectLoader::load_triangle_resources(
        &arg.obj_path,
    );

    let inner_rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();

    inner_rt.block_on(async move {
        let result = TuiApp::new(raster).run(res, None).await;
        if let Err(e) = result {
            error!("tui return an error, {}", e.to_string());
        };
    });

    info!("tui app end");

    // let tui_ms = ms.clone();
    // if arg.split {
    //     let inner_rt = tokio::runtime::Builder::new_current_thread()
    //         .enable_all()
    //         .build()
    //         .unwrap();
    //     inner_rt.block_on(async {
    //         let result = TuiSplitApp::new(tui_ms).run().await;
    //         if let Err(e) = result {
    //             error!("tui split thread error: {}", e);
    //         }
    //     });

    //     return;
    // } else {
    //     let raster_ms = ms.clone();
    //     std::thread::Builder::new().name("tui_renderer_thread".into()).spawn(move || {
    //             let camera = Camera::new(45., (WIDTH / HEIGHT) as f32,
    //                                      -5., -50., Vector3::from_xyz(0., 0., 10.),
    //                                      Vector3::from_xyz(0., 0., -1.),
    //                                      Vector3::from_xyz(0., -1., 0.));
    //             let shader = LambertianShader::new(Vector3::from_xyz(0., 1., 0.),
    //                                                0.8, 1., &camera, arg.term);
    //             let raster = RasterRunner::new(raster_ms, camera,
    //                                            Box::new(shader), arg.term);
    //             let inner_rt = tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();

    //             inner_rt.block_on(async {
    //                 let res = dognut::department::model::object_loader::ObjectLoader::load_triangle_resources(&arg.obj_path);
    //                 let _dimension = (256, 79);
    //                 let camera = self_type::camera_instance(WIDTH, HEIGHT);
    //                 let state = dognut::wgpu::wgpu_helper::State::new(winit::dpi::LogicalSize { width: WIDTH, height: HEIGHT }, camera).await;
    //                 let result = TuiWinApp::new(raster, res, tui_ms).run(Some(state));
    //                 if let Err(e) = result {
    //                     error!("tui return an error, {}", e.to_string());
    //                 };
    //             });
    //         }).unwrap();

    //     dognut::department::view::local_window::start(win_receiver);
    //     return;
    // }

    //else {
    //     let rt = tokio::runtime::Builder::new_current_thread()
    //         .enable_all()
    //         .build()
    //         .unwrap();
    //     rt.block_on(window::run(win_receiver, ms, arg.split))
    //         .expect("fail on block");
    // }
}
