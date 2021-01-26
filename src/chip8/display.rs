use sdl2::{
    event::Event, keyboard::Keycode, pixels::Color, rect::Rect, render::TextureCreator,
    video::WindowContext, Sdl,
};
use sdl2::{render::Canvas, video::Window};
use std::fmt;

pub struct Display {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<u8>,
    pub sdl_ctx: Sdl,
    pub canvas: Canvas<Window>,
    pub scale: (u32, u32),
    pub texture_creator: TextureCreator<WindowContext>,
}

pub const BASE_WIDTH: u32 = 64;
pub const BASE_HEIGHT: u32 = 32;

impl Display {
    pub fn new(width: u32, height: u32) -> Self {
        let (sdl_ctx, canvas, texture_creator) = Display::init_sdl(width, height);
        Display {
            width,
            height,
            pixels: vec![0; (BASE_WIDTH as usize) * (BASE_HEIGHT as usize)],
            sdl_ctx,
            canvas,
            scale: (width / BASE_WIDTH, height / BASE_HEIGHT),
            texture_creator,
        }
    }

    fn init_sdl(width: u32, height: u32) -> (Sdl, Canvas<Window>, TextureCreator<WindowContext>) {
        let sdl_context = sdl2::init().unwrap();
        let video_subsystem = sdl_context.video().unwrap();
        let window = video_subsystem
            .window("Chip-8 Emulator", width, height)
            .position_centered()
            .opengl()
            .build()
            .map_err(|e| e.to_string())
            .unwrap();
        let mut canvas = window.into_canvas().build().unwrap();
        let tex_creator = canvas.texture_creator();
        canvas.set_draw_color(Color::GREEN);
        (sdl_context, canvas, tex_creator)
    }

    pub fn get_pixel(&self, x: u32, y: u32) -> u8 {
        self.pixels[(y * BASE_WIDTH) as usize + x as usize]
    }

    pub fn set_pixel(&mut self, x: u32, y: u32, bit: u8) {
        self.pixels[(y * BASE_WIDTH) as usize + x as usize] = bit;
    }

    pub fn update(&mut self) -> bool {
        for event in self.sdl_ctx.event_pump().unwrap().poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => {
                    return true;
                }
                _ => {}
            }
        }
        false
    }
    pub fn clear(&mut self) {
        for pixel in self.pixels.iter_mut() {
            *pixel = 0;
        }
        self.draw(true);
    }

    pub fn draw(&mut self, is_clear: bool) {
        let mut texture = self
            .texture_creator
            .create_texture_target(
                self.texture_creator.default_pixel_format(),
                self.width,
                self.height,
            )
            .unwrap();
        let pixels = &self.pixels;
        let x_scale = &self.scale.0;
        let y_scale = &self.scale.1;
        self.canvas
            .with_texture_canvas(&mut texture, |texture_canvas| {
                texture_canvas.set_draw_color(Color::BLACK);
                texture_canvas.clear();
                if is_clear {
                    return;
                };
                for (ind, pixel) in pixels.iter().enumerate() {
                    if *pixel == 1 {
                        texture_canvas.set_draw_color(Color::YELLOW);
                        texture_canvas
                            .fill_rect(Rect::new(
                                (ind % BASE_WIDTH as usize) as i32 * (*x_scale as i32),
                                (ind / BASE_WIDTH as usize) as i32 * (*y_scale as i32),
                                *x_scale,
                                *y_scale,
                            ))
                            .unwrap();
                    }
                }
            })
            .unwrap();

        self.canvas.clear();
        self.canvas.copy(&texture, None, None).unwrap();
        self.canvas.present();
    }
}

impl fmt::Debug for Display {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SDL Display").finish()
    }
}
