use std::error::Error;
use std::io::{stdout, Stdout, Write};
use std::time::Duration;

use crossterm;
use crossterm::event::Event;
use crossterm::terminal::{disable_raw_mode, enable_raw_mode, ClearType, EnterAlternateScreen};
use crossterm::{event, execute, terminal};
use game_loop::{GameLoop, Time, TimeTrait};

use crate::department::{
    common::{
        constant::{HEIGHT, WIDTH},
        self_type,
    },
    control::camera_controller::CameraController,
    model::triangle_resources::TriangleResources,
    pipeline::rasterizer::RasterRunner,
    preview::{homo_transformation::HomoTransform, output_buffer::OutputBuffer, vector::Vector3},
    types::msg::TransferMsg,
};

pub mod term;

pub struct TuiApp {
    pub raster: RasterRunner,
    stdout: Stdout,
    theta: f32,
    camera_controller: CameraController,
    gpu: Option<self_type::StateImp>,
    is_playing_music: bool,
    music_stop_tx: Option<tokio::sync::oneshot::Sender<()>>,
}

static FPS: u32 = 30;

static TIME_STEP: Duration = Duration::from_nanos(1_000_000_000 / FPS as u64);

pub fn game_loop<G, U, R>(
    game: G,
    updates_per_second: u32,
    max_frame_time: f64,
    mut update: U,
    mut render: R,
) -> GameLoop<G, game_loop::Time, ()>
where
    U: FnMut(&mut game_loop::GameLoop<G, game_loop::Time, ()>),
    R: FnMut(&mut game_loop::GameLoop<G, game_loop::Time, ()>),
{
    let mut game_loop = game_loop::GameLoop::new(game, updates_per_second, max_frame_time, ());

    while game_loop.next_frame(&mut update, &mut render) {}

    game_loop
}

impl TuiApp {
    pub fn new(raster: RasterRunner) -> Self {
        Self {
            raster,
            stdout: stdout(),
            theta: 0.,
            gpu: None,
            camera_controller: CameraController::new(2.0, 0.2, true),
            is_playing_music: false,
            music_stop_tx: None,
        }
    }

    pub async fn run(
        mut self,
        state: Option<self_type::StateImp>,
    ) -> Result<(), Box<dyn Error>> {
        let _dimension = (256, 79);
        let camera = self_type::camera_instance(WIDTH, HEIGHT);
        let state = crate::wgpu::wgpu_helper::State::new(
            winit::dpi::LogicalSize {
                width: WIDTH,
                height: HEIGHT,
            },
            camera,
        )
        .await;
        enable_raw_mode()?;

        execute!(self.stdout, crossterm::cursor::Hide)?;
        execute!(self.stdout, EnterAlternateScreen, event::EnableMouseCapture)?;
        execute!(self.stdout, crossterm::terminal::Clear(ClearType::All))?;

        let dimension = (256, 79);
        self.gpu = Some(state);

        let _lop = game_loop(
            self,
            FPS,
            0.1,
            |g| {
                // update
                g.game.update(g.last_frame_time());
            },
            |g| {
                let mut should_exit = false;
                loop {
                    if let Ok(ready) = event::poll(Duration::from_secs(0)) {
                        if ready {
                            let event_res = event::read();
                            if event_res.is_ok() {
                                match event_res.unwrap() {
                                    Event::FocusGained => {}
                                    Event::FocusLost => {}
                                    Event::Key(k) => {
                                        if !g.game.camera_controller.process_tui_keyboard(&k) {
                                            if let Some(tx) = g.game.music_stop_tx.take() {
                                                let _ = tx.send(());
                                            }
                                            should_exit = true;
                                        }
                                        if k.code == event::KeyCode::Char('m')
                                            && !g.game.is_playing_music
                                        {
                                            g.game.is_playing_music = true;
                                            let (stop_tx, stop_rx) =
                                                tokio::sync::oneshot::channel();
                                            g.game.music_stop_tx = Some(stop_tx);

                                            tokio::spawn(async move {
                                                let (_stream, handle) =
                                                    rodio::OutputStream::try_default().unwrap();
                                                let sink = rodio::Sink::try_new(&handle).unwrap();
                                                let music = include_bytes!(
                                                    "../../../res/music/christmas.mp3"
                                                );
                                                let music_cursor = std::io::Cursor::new(music);
                                                let source =
                                                    rodio::Decoder::new(music_cursor).unwrap();

                                                sink.append(source);

                                                let mut stop_receiver = stop_rx;

                                                loop {
                                                    if sink.empty() {
                                                        break;
                                                    }

                                                    // 检查是否收到停止信号
                                                    if stop_receiver.try_recv().is_ok() {
                                                        sink.stop();
                                                        break;
                                                    }

                                                    // 短暂休眠以避免CPU过度使用
                                                    std::thread::sleep(Duration::from_millis(100));
                                                }
                                            });
                                        }
                                    }
                                    Event::Mouse(_) => {}
                                    Event::Paste(_) => {}
                                    Event::Resize(w, h) => {
                                        //println!("terminal window update to new size {} {}", w, h);
                                        //self.draw((w as u32, h as u32), &res);
                                    }
                                }
                            } else {
                                break;
                            }
                        } else {
                            break;
                        }
                    } else {
                        break;
                    }
                }

                if should_exit {
                    g.exit();
                }
                // execute!(g.game.stdout, terminal::Clear(ClearType::All));
                g.game.draw((dimension.0 as u32, dimension.1 as u32));

                let st = TIME_STEP.as_secs_f64() - Time::now().sub(&g.current_instant());
                if st > 0. {
                    std::thread::sleep(Duration::from_secs_f64(st));
                }
            },
        );
        Ok(())
    }

    pub fn update(&mut self, last_frame_time: f64) {
        if let Some(ref mut gpu) = self.gpu {
            gpu.update_outside(
                &mut self.camera_controller,
                Duration::from_secs_f64(last_frame_time),
            );
        }
    }

    pub fn draw(&mut self, dim: (u32, u32)) {
        if let Some(ref mut gpu) = self.gpu {
            let mut out_buf = OutputBuffer::new(dim.0 as u32, dim.1 as u32, true);
            out_buf.stdout = Some(&mut self.stdout);
            let out = gpu.render(true, false);
            out_buf.display.copy_from_slice(&out.0);
            //self.raster.encoder_tx.enc.send(TransferMsg::RenderPc(out)).unwrap();
            out_buf.queue_to_stdout();
            drop(out_buf);
            self.stdout.flush().unwrap();
            return;
        }
    }
}

impl Drop for TuiApp {
    fn drop(&mut self) {
        if let Some(tx) = self.music_stop_tx.take() {
            let _ = tx.send(());
        }
        let _ = execute!(self.stdout, terminal::Clear(ClearType::All));
        let _ = execute!(
            self.stdout,
            terminal::LeaveAlternateScreen,
            event::DisableMouseCapture
        );
        let _ = execute!(self.stdout, crossterm::cursor::Show);
        disable_raw_mode().unwrap();
    }
}
