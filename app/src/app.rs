use std::sync::{Arc, atomic::{AtomicBool, Ordering}};

use sdl2::{
    EventPump,

    event::{
        Event,
        WindowEvent
    },
    keyboard::Keycode,
    pixels::Color,
    render::{
        BlendMode,
        Canvas,
        Texture,
        TextureAccess,
        TextureCreator,
    },
    video::{
        Window,
        WindowContext
    },
};

static CANVAS_SCALE: f64 = 6.0;

struct RenderTarget {
    texture: Texture,
    scale: f64,
    needs_update: bool,
}

pub struct Context<'a> {
    pub events: EventPump,
    pub texture_creator: &'a TextureCreator<WindowContext>,
}

pub trait App {
    fn init(&mut self, _ctx: &mut Context) {}
    fn update(&mut self, _ctx: &mut Context) {}
    fn draw(&mut self, _ctx: &mut Context, _canvas: &mut Canvas<Window>) {}
    fn key_pressed(&mut self, _ctx: &mut Context, _keycode: Keycode) {}
    fn key_released(&mut self, _ctx: &mut Context, _keycode: Keycode) {}
}

impl RenderTarget {
    pub fn new(texture_creator: &TextureCreator<WindowContext>, window: &Canvas<Window>, scale: f64)
        -> Self
    {
        Self {
            texture: Self::create(texture_creator, window, scale),
            scale,
            needs_update: false,
        }
    }

    fn set_scale(&mut self, scale: f64) {
        self.scale = scale;
        self.needs_update = true;
    }

    fn update(&mut self, texture_creator: &TextureCreator<WindowContext>, window: &Canvas<Window>) {
        if self.needs_update {
            self.texture = Self::create(texture_creator, window, self.scale);
        }
    }

    fn create(texture_creator: &TextureCreator<WindowContext>, canvas: &Canvas<Window>, scale: f64)
        -> Texture
    {
        let (width, height) = canvas.output_size().unwrap();

        texture_creator.create_texture(
            texture_creator.default_pixel_format(),
            TextureAccess::Target,
            (width as f64 / scale) as u32,
             (height as f64 / scale) as u32
        ).unwrap()
    }
}

impl<'a> Context<'a> {
    pub fn get_mouse_pos(&mut self) -> (f64, f64) {
        let mouse = self.events.mouse_state();

        (mouse.x() as f64 / CANVAS_SCALE, mouse.y() as f64 / CANVAS_SCALE)
    }
}

pub fn run<A, F>(f: F, should_run: Arc<AtomicBool>)
    where
        A: App,
        F: FnOnce(&mut Context) -> A
{
    let sdl2_ctx = sdl2::init().unwrap();


    let video = sdl2_ctx.video().unwrap();
    let window = video.window("ZED", 1000, 1000)
        .position_centered()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas()
        .present_vsync()
        .build()
        .unwrap();

    let mut events = sdl2_ctx.event_pump().unwrap();
    let texture_creator = canvas.texture_creator();

    let mut ctx = Context {
        events,
        texture_creator: &texture_creator,
    };

    let mut app = f(&mut ctx);

    sdl2::hint::set("SDL_RENDER_SCALE_QUALITY", "0");
    let mut render_target = RenderTarget::new(&ctx.texture_creator, &canvas, CANVAS_SCALE);

    canvas.set_blend_mode(BlendMode::None);
    render_target.texture.set_blend_mode(BlendMode::None);


    app.init(&mut ctx);

    'main: while should_run.load(Ordering::Relaxed) {
        'events: loop {
            let ev = ctx.events.poll_event();
            match ev {
                None => break 'events,
                Some(ev) => {
                    match ev {
                        Event::KeyDown { keycode: Some(Keycode::Escape), .. }
                        | Event::Window { win_event: WindowEvent::Close, .. } => {
                            should_run.store(false, Ordering::Relaxed);
                            break 'main;
                        },

                        Event::KeyDown { keycode: Some(keycode), .. } => {
                            app.key_pressed(&mut ctx, keycode);
                        },
                        Event::KeyUp { keycode: Some(keycode), .. } => {
                            app.key_released(&mut ctx, keycode);
                        }
                        _ => ()
                    }
                }
            }
        }

        app.update(&mut ctx);

        canvas.set_draw_color(Color::RGB(85, 117, 139));
        canvas.clear();

        canvas.with_texture_canvas(&mut render_target.texture,|canvas| {
            canvas.clear();
            app.draw(&mut ctx, canvas);
        });
        canvas.set_draw_color(Color::RGB(255, 255, 255));
        canvas.copy(&render_target.texture, None, None);
        canvas.present();
    }


}
