// use crate::department::common::constant::{HEIGHT, WIDTH};
// use crate::department::types::msg::{DognutOption, TransferMsg};
// use crate::department::types::multi_sender::MultiSender;
// use crossbeam_channel::Receiver;
// use log::{debug, info};
// use pixels::wgpu::Color;
// use pixels::{Pixels, SurfaceTexture};
// use winit::dpi::LogicalSize;
// use winit::event_loop::EventLoop;

// pub struct LocalWindow {
//     window: winit::window::Window,
//     pixels: Pixels,
//     id: winit::window::WindowId,
//     win_rgba_rx: Receiver<TransferMsg>,
//     encoder_start_working: bool,
//     ev_loop: EventLoop<()>,
// }

// impl LocalWindow {
//     pub fn new(
//         window: winit::window::Window,
//         pixels: Pixels,
//         id: winit::window::WindowId,
//         win_rgba_rx: Receiver<TransferMsg>,
//         ev_loop: EventLoop<()>,
//     ) -> Self {
//         Self {
//             window,
//             pixels,
//             id,
//             win_rgba_rx,
//             encoder_start_working: false,
//             ev_loop,
//         }
//     }

//     pub fn run(mut self) {
//         let ev_loop = EventLoop::new().unwrap();

//         self.ev_loop.run(move |_event, _, control_flow| {
//             // match event {
//             //     winit::event::Event::WindowEvent { event, .. } => match event {
//             //         winit::event::WindowEvent::CloseRequested => {
//             //             *control_flow = winit::event_loop::ControlFlow::Exit;
//             //         }
//             //         winit::event::WindowEvent::Resized(size) => {
//             //
//             //         }
//             //         _ => {}
//             //     },
//             //     winit::event::Event::MainEventsCleared => {
//             //         self.window.request_redraw();
//             //     }
//             //     winit::event::Event::RedrawRequested(_) => {
//             //     }
//             //     _ => {}
//             // }
//             let msg = self.win_rgba_rx.recv().unwrap();
//             match msg {
//                 TransferMsg::RenderedData(rgba) => {
//                     self.pixels.get_frame_mut().copy_from_slice(&rgba);
//                     self.pixels.render().expect("should render success");
//                 }
//                 TransferMsg::QuitThread => {
//                     *control_flow = winit::event_loop::ControlFlow::Exit;
//                     debug!("LocalWindow::run() end");
//                     return;
//                 }
//                 _ => {
//                     *control_flow = winit::event_loop::ControlFlow::Exit;
//                 }
//             }
//         });
//     }
// }

// pub fn start(win_rgba_rx: Receiver<TransferMsg>) {
//     let event_loop = EventLoop::new();

//     let window = {
//         let size = LogicalSize::new(WIDTH as f64, HEIGHT as f64);
//         WindowBuilder::new()
//             .with_title("Main Window")
//             .with_inner_size(size)
//             .with_min_inner_size(size)
//             .build(&event_loop)
//             .unwrap()
//     };

//     window.set_outer_position(winit::dpi::Position::from(winit::dpi::PhysicalPosition {
//         x: 100,
//         y: 100,
//     }));

//     let id = window.id();
//     let scale_factor = window.scale_factor();

//     let mut pixels = {
//         let window_size = window.inner_size();
//         info!("scale factor is {}", scale_factor);
//         let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, &window);
//         Pixels::new(WIDTH, HEIGHT, surface_texture).unwrap()
//     };
//     pixels.clear_color(Color::WHITE);
//     let lw = LocalWindow::new(window, pixels, id, win_rgba_rx, event_loop);
//     lw.run();
// }
