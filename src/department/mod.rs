use gilrs::GamepadId;
use gilrs::Gilrs;
use pixels::Pixels;
use winit::window::WindowId;
use winit_input_helper::WinitInputHelper;

use crate::department::common::self_type;
use crate::department::control::camera_controller::CameraController;

pub mod common;
pub mod control;
pub mod model;
pub mod net;
pub mod pipeline;
pub mod preview;
pub mod tui;
pub mod types;
pub mod view;
