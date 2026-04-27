use std::{array, sync::{Arc, RwLock}};

use sdl3::{self, EventPump, GamepadSubsystem, Sdl, event::Event, gamepad::{Axis, Button, Gamepad}, sensor::SensorType, sys::joystick::SDL_JoystickID};

pub struct SdlContainer {
    sdl: Sdl,
    gp_sys: GamepadSubsystem,
    event_pump: EventPump,
    controllers: [(Option<Gamepad>, Arc<RwLock<ControllerState>>); Self::NUM_PLAYERS],
}

impl SdlContainer {
    const NUM_PLAYERS: usize = 4;

    pub fn new() -> Self {
        let sdl = sdl3::init().unwrap();
        let gp_sys = sdl.gamepad().unwrap();
        let event_pump = sdl.event_pump().unwrap();
        SdlContainer { 
            sdl,
            gp_sys,
            event_pump,
            controllers: array::from_fn(|_| (None, Arc::new(RwLock::new(ControllerState::new())))),
        }
    }

    pub fn tick(&mut self) {
        // Check for controller connections/disconnections
        self.event_pump.pump_events();

        for event in self.event_pump.poll_iter() {
            match event {
                Event::ControllerDeviceAdded { timestamp, which } => {
                    // Open gamepad and enable all sensors
                    let mut gamepad = self.gp_sys.open(SDL_JoystickID(which)).unwrap();
                    let player_index = gamepad.player_index().unwrap();
                    if (0..Self::NUM_PLAYERS as u16).contains(&player_index) {
                        for sensor in ControllerState::SENSOR_MAPPING {
                            // Enable all sensors possible; doesn't matter if it fails
                            let _ = gamepad.sensor_set_enabled(sensor, true);
                        }
                        // Just a connection indicator; doesn't matter if it fails
                        let _ = gamepad.set_rumble(0xFFFF, 0xFFFF, 100);
                        self.controllers[player_index as usize].0 = Some(gamepad);
                        println!("Controller P{} connected", player_index + 1);
                    } else {
                        println!("Controller connection failed: Player index {} out of bounds", player_index + 1);  
                    }
                }
                Event::ControllerDeviceRemoved { timestamp, which } => {
                    // Find gamepad in controller array via its joystick ID, then remove it
                    if let Some(player_index) = self.controllers.iter().position(|(optgp, _)| optgp.as_ref().is_some_and(|gp| gp.id().is_ok_and(|id| id.0 == which))) {
                        self.controllers[player_index].0 = None;
                        self.controllers[player_index].1.write().unwrap().reset();
                        println!("Controller P{} disconnected", player_index + 1);
                    }
                }
                _ => {}
            }
        }

        // Poll all inputs and update internal state
        for (optgp, state) in self.controllers.iter() {
            let Some(gamepad) = optgp else {continue;};
            state.write().unwrap().update(gamepad);
        }
    }

    pub fn clone_p1(&self) -> Arc<RwLock<ControllerState>> {
        self.controllers[0].1.clone()
    }
}

pub struct ControllerState {
    buttons: [bool; Self::NUM_BUTTONS],
    axes: [i16; Self::NUM_AXES],
    sensors: [[f32; 3]; Self::NUM_SENSORS],
}

impl ControllerState {
    const NUM_BUTTONS: usize = 25;
    const BUTTON_MAPPING: [Button; Self::NUM_BUTTONS] = [
        Button::South,
        Button::East,
        Button::West,
        Button::North,
        Button::Back,
        Button::Guide,
        Button::Start,
        Button::LeftStick,
        Button::RightStick,
        Button::LeftShoulder,
        Button::RightShoulder,
        Button::DPadUp,
        Button::DPadDown,
        Button::DPadLeft,
        Button::DPadRight,
        Button::Misc1,
        Button::RightPaddle1,
        Button::LeftPaddle1,
        Button::RightPaddle2,
        Button::LeftPaddle2,
        Button::Touchpad,
        Button::Misc2,
        Button::Misc3,
        Button::Misc4,
        Button::Misc5,
    ];
    const fn button_index(button: Button) -> usize {
        match button {
            Button::South => 0,
            Button::East => 1,
            Button::West => 2,
            Button::North => 3,
            Button::Back => 4,
            Button::Guide => 5,
            Button::Start => 6,
            Button::LeftStick => 7,
            Button::RightStick => 8,
            Button::LeftShoulder => 9,
            Button::RightShoulder => 10,
            Button::DPadUp => 11,
            Button::DPadDown => 12,
            Button::DPadLeft => 13,
            Button::DPadRight => 14,
            Button::Misc1 => 15,
            Button::RightPaddle1 => 16,
            Button::LeftPaddle1 => 17,
            Button::RightPaddle2 => 18,
            Button::LeftPaddle2 => 19,
            Button::Touchpad => 20,
            Button::Misc2 => 21,
            Button::Misc3 => 22,
            Button::Misc4 => 23,
            Button::Misc5 => 24,
        }
    }

    const NUM_AXES: usize = 6;
    const STICK_THRESHOLD: i16 = 4000;
    const AXIS_MAPPING: [Axis; Self::NUM_AXES] = [
        Axis::LeftX,
        Axis::LeftY,
        Axis::RightX,
        Axis::RightY,
        Axis::TriggerLeft,
        Axis::TriggerRight,
    ];

    const NUM_SENSORS: usize = 6;
    const SENSOR_MAPPING: [SensorType; Self::NUM_SENSORS] = [
        SensorType::Gyroscope,
        SensorType::Accelerometer,
        SensorType::AccelerometerLeft,
        SensorType::AccelerometerRight,
        SensorType::GyroscopeLeft,
        SensorType::GyroscopeRight,
    ];

    pub fn new() -> Self {
        ControllerState {
            buttons: [false; Self::NUM_BUTTONS], 
            axes: [0; Self::NUM_AXES], 
            sensors: [[0.0; 3]; Self::NUM_SENSORS] 
        }
    }

    pub fn update(&mut self, gamepad: &Gamepad) {
        // Poll all buttons
        for (index, button) in Self::BUTTON_MAPPING.iter().enumerate() {

            self.buttons[index] = gamepad.button(*button);
        }

        // Poll all axes
        for (index, axis) in Self::AXIS_MAPPING.iter().enumerate() {
            self.axes[index] = gamepad.axis(*axis);
        }

        // Poll all sensors
        for (index, sensor) in Self::SENSOR_MAPPING.iter().enumerate() {
            // An error most likely means the sensor is disabled and can be ignored
            let _ = gamepad.sensor_get_data(*sensor, &mut self.sensors[index]);
        }
    }

    pub fn reset(&mut self) {
        self.buttons.fill(false);
        self.axes.fill(0);
        self.sensors.fill([0.0; 3]);
    }

    pub fn poll_button(&self, button: Button) -> bool {
        self.buttons[Self::button_index(button)]
    }

    pub fn poll_direction(&self, direction: Direction) -> bool {
        // kinda ugly but it works
        let (button_index, axis_index, axis_sign) = match direction {
            Direction::Up => (11, 1, -1),
            Direction::Down => (12, 1, 1),
            Direction::Left => (13, 0, -1),
            Direction::Right => (14, 0, 1),
        };

        let button_active = self.buttons[button_index];
        let axis_active = self.axes[axis_index] * axis_sign > Self::STICK_THRESHOLD;
        button_active || axis_active
    }
}

pub enum Direction {
    Up,
    Down,
    Left,
    Right
}