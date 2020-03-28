use app::{
    app::{run, App, Context},
    controller
};
use bottles::{Dispatcher, Queue};

use zed_shared::protocol::{SimpleProtocol, Protocol};
use zed_shared::message;

use laminar::{Socket, Packet, SocketEvent};
use legion::entity::Entity;
use legion::query::{Read, Write, IntoQuery, Tagged};
use legion::filter::filter_fns::tag;
use legion::borrow::RefMut;

use sdl2::{
    image::LoadTexture,
    keyboard::Keycode,
    pixels::Color,
    rect::Rect,
    rect::Point,
    render::{
        Canvas,
        Texture,
    },
    video::{
        Window,
    }
};
use std::collections::HashMap;
use std::collections::hash_map::Entry;
use std::sync::{Arc, Mutex};
use std::rc::Rc;
use rand::random;
use std::cell::RefCell;

use crossbeam_channel::{Sender, Receiver};
use std::net::SocketAddr;

use serde::{Serialize, de::DeserializeOwned};


#[derive(Clone, Copy, PartialEq)]
struct Position {
    x: f64,
    y: f64
}

#[derive(Clone, Copy, PartialEq)]
struct Direction {
    x: f64,
    y: f64
}

impl Direction {
    pub fn angle(&self) -> f64 {
        self.y.atan2(self.x)
    }
    
    pub fn set_angle(&mut self, angle: f64) {
        self.x = angle.cos();
        self.y = angle.sin();
    }
}

enum HitBox {
    Circle { radius: f64 },
    Box { width: f64, height: f64 }
}

#[derive(Clone, PartialEq)]
struct Model {
    texture_static: String,
    texture_blend: String,
    color: Color,
    frame_width: usize,
    frame_height: usize,
    frame: (usize, usize)
}

#[derive(Clone, Copy, PartialEq)]
struct Player {
    holster: bool,
    id: Option<usize>,
}

#[derive(Clone, Copy, PartialEq)]
struct LocalPlayer {}

pub struct Net {
    sender: Sender<Packet>,
    receiver: Receiver<SocketEvent>,
    addr: SocketAddr,
    protocol: RefCell<SimpleProtocol>,
    queue: RefCell<Queue<Main>>,
}

pub struct Main {
    controller: controller::Controller,
    counter: f64,
    textures: HashMap<String, Texture>,
    ecs: legion::world::World,

    local_player_id: Option<usize>,

    net: Rc<Net>,
    others: Arc<Mutex<HashMap<usize, Entity>>>
}

impl Net {
    pub fn new(socket: &mut Socket, addr: SocketAddr, protocol: SimpleProtocol) -> Self {
        Self {
            sender: socket.get_packet_sender(),
            receiver: socket.get_event_receiver(),
            addr,
            protocol: RefCell::new(protocol),
            queue: RefCell::new(Queue::new()),
        }
    }

    pub fn register<T: 'static + DeserializeOwned>(&self) {
        self.queue.borrow_mut().register::<T>(self.protocol.borrow_mut().dispatcher_mut());
    }

    pub fn subscribe<M, F>(&self, f: F)
        where
            F: FnMut(&mut Main, Rc<M>) + 'static,
            M: 'static,
    {
        self.queue.borrow_mut().subscribe(self.protocol.borrow_mut().dispatcher_mut(), f);
    }

    pub fn send_reliable_unordered<T: 'static + Serialize>(&self, message: T) {
        self.protocol.borrow_mut().send_reliable_unordered(&self.sender, self.addr, message);
    }

    pub fn poll(main: &mut Main) {
        while let Ok(event) = main.net.receiver.try_recv() {
            match event {
                SocketEvent::Connect(addr) => {
                    println!("Connected to {}", addr);
                },
                SocketEvent::Timeout(addr) => {
                    println!("Timeout of {}", addr);
                },
                SocketEvent::Packet(packet) => {
                    main.net.protocol.borrow_mut().receive(packet.payload());
                },
                _ => ()
            }
        }

        let net = Rc::clone(&main.net);
        let mut queue = net.queue.borrow_mut();

        queue.poll(main);
    }
}

impl Main {
    pub fn new(ctx: &mut Context, net: Net) -> Self {
        let mut images = HashMap::new();
        images.insert("guy_static".into(), ctx.texture_creator.load_texture("static/guy_static.png").unwrap());
        images.insert("guy_blend".into(), ctx.texture_creator.load_texture("static/guy_blend.png").unwrap());

        let mut ecs = legion::world::World::new();
        ecs.insert((LocalPlayer {},),
[(
            Position { x: 16.0, y: 16.0 },
            Direction { x: 0.0, y: 0.0 },
            Model {
                texture_static: "guy_static".into(),
                texture_blend: "guy_blend".into(),
                color: Color::RGB(random(), random(), random()),
                frame_width: 20,
                frame_height: 20,
                frame: (1, 1)
            },
            Player {
                id: None,
                holster: false,
            }
        )].iter().cloned());

        Self {
            counter: 0.0,
            controller: controller::Controller::new(),
            textures: images,
            ecs: ecs,
            net: Rc::new(net),
            others: Arc::new(Mutex::new(HashMap::new())),
            
            local_player_id: None,
        }
    }

    fn control_player(&mut self, ctx: &mut Context) {
        let dt = 1.0/60.0;
        let spd = 50.0;

        for (mut position) in <(Write<Position>)>::query().filter(tag::<LocalPlayer>()).iter(&mut self.ecs) {
            let mut dx: f64 = 0.0;
            let mut dy: f64 = 0.0;

            if self.controller.left.pressed() {
                dx -= 1.0;
            }
            if self.controller.right.pressed() {
                dx += 1.0;
            }
            if self.controller.up.pressed() {
                dy -= 1.0;
            }
            if self.controller.down.pressed() {
                dy += 1.0;
            }

            let len = (dx.powi(2) + dy.powi(2)).sqrt();

            if len > 0.0 {
                dx /= len;
                dy /= len;

                position.x += dx * dt * spd;
                position.y += dy * dt * spd;
            }
        }

        let (mx, my) = ctx.get_mouse_pos();
        for (mut direction, position) in <(Write<Direction>, Read<Position>)>::query().filter(tag::<LocalPlayer>())
            .iter(&mut self.ecs)
        {
            direction.x = mx - position.x;
            direction.y = my - position.y;
        }


        for (mut model, player) in <(Write<Model>, Read<Player>)>::query().iter(&mut self.ecs) {
            model.frame.0 = if player.holster { 1 } else { 0 };
        }
    }

    fn draw_models(&mut self, ctx: &mut Context, canvas: &mut Canvas<Window>) {
        use std::f64::consts::PI;


        for (position, direction, model) in <(Read<Position>, Read<Direction>, Read<Model>)>::query().iter(&mut self.ecs) {
            let angle = direction.angle();

            let frame = crate::util::calculate_frame_from_angle(angle, 4);
            let angle_deg = frame.1 * 180.0 / PI;

            let (frame_x, frame_y) = model.frame;
            let frame_y = frame_y*4 + frame.0;

            let x = position.x;
            let y = position.y;

            let qs = Rect::new(
                (model.frame_width * frame_x) as i32,
                (model.frame_height * (frame_y)) as i32,
                model.frame_width as u32,
                model.frame_height as u32
            );
            let qd = Rect::new(
                x as i32 - model.frame_width as i32 / 2,
                y as i32 - model.frame_height as i32 / 2,
                model.frame_width as u32,
                model.frame_height as u32
            );

            let img_static = self.textures.get(&model.texture_static).unwrap();
            canvas.set_draw_color(Color::RGB(255, 255, 255));
            canvas.copy_ex(
                img_static,
                qs,
                qd,
                angle_deg,
                Point::new(model.frame_width as i32 / 2, model.frame_height as i32 / 2),
                false,
                false)
                .unwrap();

            let img_blend = self.textures.get_mut(&model.texture_blend).unwrap();
            img_blend.set_color_mod(model.color.r, model.color.g, model.color.b);
            canvas.copy_ex(img_blend, qs, qd, angle_deg,
                           Point::new(model.frame_width as i32 / 2, model.frame_height as i32 / 2),
                           false, false)
                .unwrap();

            canvas.set_draw_color(Color::RGB(255, 255, 255));

            let (c, s) = (angle.cos(), angle.sin());
            canvas.fill_rect(Rect::new((x + c*16.0) as i32, (y + s*16.0) as i32, 1, 1)).unwrap();
            canvas.fill_rect(Rect::new((x + c*17.0) as i32, (y + s*17.0) as i32, 1, 1)).unwrap();
            canvas.fill_rect(Rect::new((x + c*18.0) as i32, (y + s*18.0) as i32, 1, 1)).unwrap();
        }
    }

    fn send_player_pos(&mut self) {
        let local_player_id = match self.local_player_id {
            None => return,
            Some(id) => id
        };

        use zed_shared::message::both::PlayerStatus;
        
        let q = <(Read<Position>, Read<Direction>, Read<Model>, Read<Player>)>::query().filter(tag::<LocalPlayer>());
        for (pos, dir, model, player) in q.iter(&mut self.ecs) {
            let status = PlayerStatus {
                x: pos.x,
                y: pos.y,
                angle: dir.angle(),
                player_id: local_player_id,
                r: model.color.r,
                g: model.color.g,
                b: model.color.b,
                holster: player.holster, 
            };
            
            self.net.send_reliable_unordered(status);
        }
    }

    fn receive_greeting(&mut self, greeting: Rc<message::from_client::Greeting>) {
        println!("Received greeting from {}", greeting.name);
    }

    fn receive_greeting_response(&mut self, response: Rc<message::from_server::GreetingResponse>) {
        println!("Connected to server with player_id {}", response.player_id);
        self.local_player_id = Some(response.player_id);
    }

    fn receive_player_status(&mut self, message: Rc<message::both::PlayerStatus>) {
        let mut others = self.others.lock().unwrap();

        match self.local_player_id {
            Some(id) if message.player_id == id => return,
            _ => ()
        };
         
        match others.get(&message.player_id) {
            None => {
                let entities = self.ecs.insert((), [(
                        Position { x: message.x, y: message.y },
                        Direction { x: 0.0, y: 0.0 },
                        Model {
                            texture_static: "guy_static".into(),
                            texture_blend: "guy_blend".into(),
                            color: Color::RGB(message.r, message.g, message.b),
                            frame_width: 20,
                            frame_height: 20,
                            frame: (1, 1)
                        },
                        Player {
                            id: Some(message.player_id),
                            holster: message.holster,
                        }
                    )].iter().cloned());
                    
                println!("Creating networked player {}", message.player_id);
                others.insert(message.player_id, entities[0]);
            },
            Some(&entity) => {
                println!("Updating networked player {}", message.player_id);
                {
                    dbg!(message.x, message.y);
                    let mut pos = self.ecs.get_component_mut::<Position>(entity).unwrap();
                    pos.x = message.x;
                    pos.y = message.y;
                }
                {
                    dbg!(message.angle);
                    let mut dir = self.ecs.get_component_mut::<Direction>(entity).unwrap();
                    dir.set_angle(message.angle);
                }
                {
                    dbg!(message.r, message.g, message.b);
                    let mut model = self.ecs.get_component_mut::<Model>(entity).unwrap();
                    model.color = Color::RGB(message.r, message.g, message.b);
                }
            }
        };
    }
}

impl App for Main {
    fn init(&mut self, ctx: &mut Context) {
        self.net.register::<message::from_server::GreetingResponse>();
        self.net.subscribe(Self::receive_greeting_response);

        self.net.register::<message::from_client::Greeting>();
        self.net.subscribe(Self::receive_greeting);

        self.net.register::<message::both::PlayerStatus>();
        self.net.subscribe(Self::receive_player_status);

        println!("Sending greeting.");
        self.net.send_reliable_unordered(
            message::from_client::Greeting {
                name: "Marcin Szymczak".into(),
            }
        );
    }

    fn update(&mut self, ctx: &mut Context) {
        self.counter += 1.0/60.0;
        self.control_player(ctx);
        self.send_player_pos();
        
        Net::poll(self);
    }

    fn key_pressed(&mut self, ctx: &mut Context, keycode: Keycode) {
        match keycode {
            Keycode::A => self.controller.left.update(true),
            Keycode::D => self.controller.right.update(true),
            Keycode::S => self.controller.down.update(true),
            Keycode::W => self.controller.up.update(true),

            Keycode::H => {
                for mut player in Write::<Player>::query().filter(tag::<LocalPlayer>()).iter(&mut self.ecs) {
                    player.holster = !player.holster;
                }
            },

            Keycode::C => {
                use rand::random;
                for (mut model) in <(Write<Model>)>::query().filter(tag::<LocalPlayer>()).iter(&mut self.ecs) {
                    model.color = Color::RGB(random(), random(), random());
                }
            }
            _ => ()
        }
    }

    fn key_released(&mut self, ctx: &mut Context, keycode: Keycode) {
        match keycode {
            Keycode::A => self.controller.left.update(false),
            Keycode::D => self.controller.right.update(false),
            Keycode::S => self.controller.down.update(false),
            Keycode::W => self.controller.up.update(false),
            _ => ()
        }
    }

    fn draw(&mut self, ctx: &mut Context, canvas: &mut Canvas<Window>) {
        self.draw_models(ctx, canvas);
    }
}
