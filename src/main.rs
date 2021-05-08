use cgmath::Matrix3;
use engine3d::{
    camera::*,
    collision,
    geom::*,
    render::{InstanceGroups, InstanceRaw},
    run, Engine, DT,
};
use rand;
use rand::Rng;
use winit;
use winit::event::VirtualKeyCode as KeyCode;

const G: f32 = 1.0;
const MBHS: f32 = 0.5; // menu box half size
const WBHS: f32 = 1.0; // wall box half size
const PBHS: f32 = 0.5; // player box half size
const WH: i8 = 3; // wall height in boxes
const WW: i8 = 6; // wall width in boxes

enum Mode {
    Menu,
    GamePlay,
    EndScreen,
}

#[derive(Clone, PartialEq, Debug)]
pub struct MenuObject {
    pub body: Box,
}

impl MenuObject {
    fn render(&self, rules: &GameData, igs: &mut InstanceGroups) {
        igs.render(
            rules.menu_object_model,
            InstanceRaw {
                model: (
                    Mat4::from_translation(self.body.c.to_vec())
                        * Mat4::from_nonuniform_scale(
                            self.body.half_sizes.x,
                            self.body.half_sizes.y,
                            self.body.half_sizes.z,
                        )
                    // // * Mat4::from_scale(self.body.r)
                    // Mat4::from_nonuniform_scale(0.5, 0.05, 0.5)
                )
                .into(),
            },
        );
    }
}

#[derive(Clone, PartialEq, Debug)]
pub struct Wall {
    pub body: Vec<Box>,
    // pub velocity: Vec3,
    pub vels: Vec<Vec3>,
    control: (i8, i8),
}

impl Wall {
    pub fn generate_components(init_z: f32, axes: Mat3) -> Vec<Box> {
        let mut rng = rand::thread_rng();
        let missing_x = rng.gen_range(0..WW);
        let missing_y = rng.gen_range(0..WH);

        let mut boxes = vec![];
        let half_sizes = Vec3::new(WBHS, WBHS, WBHS);
        for x in 0..WW {
            for y in 0..WH {
                if x != missing_x || y != missing_y {
                    let c = Pos3::new(
                        x as f32 * 2.1 * WBHS + WBHS - WW as f32 * WBHS,
                        y as f32 * 2.0 * WBHS + WBHS,
                        init_z,
                    );
                    boxes.push(Box {
                        c,
                        axes,
                        half_sizes,
                    })
                }
            }
        }
        boxes
    }

    fn render(&self, rules: &GameData, igs: &mut InstanceGroups) {
        for b in &self.body {
            igs.render(
                rules.wall_model,
                InstanceRaw {
                    model: (
                        Mat4::from_translation(b.c.to_vec())
                            * Mat4::from_nonuniform_scale(
                                b.half_sizes.x,
                                b.half_sizes.y,
                                b.half_sizes.z,
                            )
                        // // * Mat4::from_scale(self.body.r)
                        // Mat4::from_nonuniform_scale(0.5, 0.05, 0.5)
                    )
                    .into(),
                },
            );
        }
    }

    fn input(&mut self, events: &engine3d::events::Events) {
        self.control.0 = if events.key_held(KeyCode::A) {
            -1
        } else if events.key_held(KeyCode::D) {
            1
        } else {
            0
        };
        self.control.1 = if events.key_held(KeyCode::W) {
            -1
        } else if events.key_held(KeyCode::S) {
            1
        } else {
            0
        };
    }

    fn integrate(&mut self) {
        for (b, v) in &mut self.body.iter_mut().zip(self.vels.iter()) {
            b.c += v * DT;
        }
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Platform {
    pub body: Plane,
    control: (i8, i8),
}

impl Platform {
    pub fn generate_bounds(wall_height: i8, wall_width: i8) -> Vec<Platform> {
        let mut bounds = vec![];
        let btn = Vec3::new(0.0, 1.0, 0.0); // bottom & top normal vector
        let lrn = Vec3::new(1.0, 0.0, 0.0); // left & right normal vector

        let top_dist = wall_height as f32 * WBHS * 2.0;
        let left_dist = wall_width as f32 * WBHS * 2.0;
        let right_dist = -1.0 * left_dist;

        let b = Platform {
            body: Plane { n: btn, d: 0.0 },
            control: (0, 0),
        };
        let t = Platform {
            body: Plane {
                n: btn,
                d: top_dist,
            },
            control: (0, 0),
        };
        let l = Platform {
            body: Plane {
                n: lrn,
                d: left_dist,
            },
            control: (0, 0),
        };
        let r = Platform {
            body: Plane {
                n: lrn,
                d: right_dist,
            },
            control: (0, 0),
        };

        bounds.push(b);
        bounds.push(t);
        bounds.push(l);
        bounds.push(r);
        bounds
    }

    fn render(&self, rules: &GameData, igs: &mut InstanceGroups) {
        igs.render(
            rules.platform_model,
            engine3d::render::InstanceRaw {
                model: (Mat4::from(cgmath::Quaternion::between_vectors(
                    Vec3::new(0.0, 1.0, 0.0),
                    self.body.n,
                )) * Mat4::from_translation(self.body.n * self.body.d)
                    * Mat4::from_translation(Vec3::new(0.0, -0.025, 0.0))
                    * Mat4::from_nonuniform_scale(0.5, 0.05, 0.5))
                .into(),
            },
        );
    }

    fn input(&mut self, events: &engine3d::events::Events) {
        self.control.0 = if events.key_held(KeyCode::A) {
            -1
        } else if events.key_held(KeyCode::D) {
            1
        } else {
            0
        };
        self.control.1 = if events.key_held(KeyCode::W) {
            -1
        } else if events.key_held(KeyCode::S) {
            1
        } else {
            0
        };
    }

    fn integrate(&mut self) {
        self.body.n += Vec3::new(
            self.control.0 as f32 * 0.4 * DT,
            0.0,
            self.control.1 as f32 * 0.4 * DT,
        );
        self.body.n = self.body.n.normalize();
    }
}

struct Game<Cam: Camera> {
    start: MenuObject,
    scores: MenuObject,
    play_again: MenuObject,
    wall: Wall,
    floor: Platform,
    // bounds: Vec<Platform>,
    player: Player,
    camera: Cam,
    wall_velocity: Vec3,
    ps: Vec<collision::Contact<usize>>,
    ww: Vec<collision::Contact<usize>>,
    pw: Vec<collision::Contact<usize>>,
    fw: Vec<collision::Contact<usize>>,
    pf: Vec<collision::Contact<usize>>,
    mode: Mode,
}

struct GameData {
    wall_model: engine3d::assets::ModelRef,
    platform_model: engine3d::assets::ModelRef,
    player_model: engine3d::assets::ModelRef,
    camera_model: engine3d::assets::ModelRef,
    menu_object_model: engine3d::assets::ModelRef,
}

#[derive(Clone, Debug)]
pub struct Player {
    pub body: Box,
    pub velocity: Vec3,
    pub acc: Vec3,
    pub rot: Quat,
    pub omega: Vec3,
}

impl Player {
    const MAX_SPEED: f32 = 3.0;
    fn render(&self, rules: &GameData, igs: &mut InstanceGroups) {
        igs.render(
            rules.player_model,
            InstanceRaw {
                model: (
                    // Mat4::from_translation(self.body.c.to_vec() - Vec3::new(0.0, 0.2, 0.0))
                    Mat4::from_translation(self.body.c.to_vec())
                        * Mat4::from_nonuniform_scale(
                            self.body.half_sizes.x,
                            self.body.half_sizes.y,
                            self.body.half_sizes.z,
                        )
                    // * Mat4::from(self.rot)
                )
                .into(),
            },
        );
    }
    fn integrate(&mut self) {
        self.velocity += ((self.rot * self.acc) + Vec3::new(0.0, -G, 0.0)) * DT;
        if self.velocity.magnitude() > Self::MAX_SPEED {
            self.velocity = self.velocity.normalize_to(Self::MAX_SPEED);
        }
        self.body.c += self.velocity * DT;
        self.rot += 0.5 * DT * Quat::new(0.0, self.omega.x, self.omega.y, self.omega.z) * self.rot;
    }
}

impl<C: Camera> engine3d::Game for Game<C> {
    type StaticData = GameData;
    fn start(engine: &mut Engine) -> (Self, Self::StaticData) {
        use rand::Rng;

        let axes = Matrix3::new(0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0);

        // create menu objects
        let menu_object_half_sizes = Vec3::new(MBHS, MBHS, MBHS);
        let start = MenuObject {
            body: Box {
                c: Pos3::new(3.0, MBHS, 0.0),
                axes,
                half_sizes: menu_object_half_sizes,
            },
        };
        let scores = MenuObject {
            body: Box {
                c: Pos3::new(-3.0, MBHS, 0.0),
                axes,
                half_sizes: menu_object_half_sizes,
            },
        };
        let play_again = MenuObject {
            body: Box {
                c: Pos3::new(3.0, MBHS, 0.0),
                axes,
                half_sizes: menu_object_half_sizes,
            },
        };

        // create wall
        let wall_init_z = 20.0;
        let wall_init_velocity = Vec3::new(0.0, 0.0, -1.5);

        // generate wall components
        let boxes = Wall::generate_components(wall_init_z, Matrix3::one());
        let n_boxes = boxes.len();
        let wall = Wall {
            body: boxes,
            vels: vec![wall_init_velocity; n_boxes],
            control: (0, 0),
        };

        // create platform
        let floor = Platform {
            body: Plane {
                n: Vec3::new(0.0, 1.0, 0.0),
                d: 0.0,
            },
            control: (0, 0),
        };

        // let bounds = Platform::generate_bounds(wall_height, wall_width);

        // create player
        let player = Player {
            body: Box {
                c: Pos3::new(0.0, PBHS, 0.0),
                axes: Matrix3::one(),
                half_sizes: Vec3::new(PBHS, PBHS, PBHS),
            },
            velocity: Vec3::zero(),
            acc: Vec3::zero(),
            omega: Vec3::zero(),
            rot: Quat::new(1.0, 0.0, 0.0, 0.0),
        };

        // create camera
        let camera = C::new(player.body.c);
        let rng = rand::thread_rng();

        // models
        // TODO: update .obj and .mtl files
        let menu_object_model = engine.load_model("box.obj");
        let wall_model = engine.load_model("box.obj");
        let floor_model = engine.load_model("floor.obj");
        let player_model = engine.load_model("cube.obj");
        let camera_model = engine.load_model("sphere.obj");

        // create game
        (
            Self {
                start,
                scores,
                play_again,
                wall,
                floor,
                // bounds,
                player,
                camera,
                wall_velocity: wall_init_velocity,
                ps: vec![],
                ww: vec![],
                fw: vec![],
                pw: vec![],
                pf: vec![],
                mode: Mode::Menu,
            },
            GameData {
                menu_object_model,
                wall_model,
                platform_model: floor_model,
                player_model,
                camera_model,
            },
        )
    }

    fn render(&self, rules: &Self::StaticData, igs: &mut InstanceGroups) {
        // always render player and floor
        self.player.render(rules, igs);
        self.floor.render(rules, igs);

        match self.mode {
            Mode::Menu => {
                self.start.render(rules, igs);
                self.scores.render(rules, igs);
            }
            Mode::GamePlay => {
                self.wall.render(rules, igs);
            }
            Mode::EndScreen => {
                self.play_again.render(rules, igs);
                self.scores.render(rules, igs);
            }
        }
    }

    fn handle_collision(&mut self) {
        self.pf.clear();
        let mut pb = [self.player.body];
        let mut pv = [self.player.velocity];

        match self.mode {
            Mode::Menu => {
                self.ps.clear();

                // collision between player and start object
                collision::gather_contacts_ab(&pb, &[self.start.body], &mut self.ps);

                // collision between player and floor
                collision::gather_contacts_ab(&pb, &[self.floor.body], &mut self.pf);

                // restitute between player and floor
                collision::restitute_dyn_stat(&mut pb, &mut pv, &[self.floor.body], &mut self.pf);

                // if player hits start menu object, start game
                if !self.ps.is_empty() {
                    // reset player position
                    self.player.body.c = Pos3::new(0.0, PBHS, 0.0);
                    self.mode = Mode::GamePlay;
                } else {
                    self.player.body = pb[0];
                    self.player.velocity = pv[0];
                }
            }
            Mode::GamePlay => {
                self.ww.clear();
                self.pw.clear();
                self.fw.clear();

                // collision between wall and wall
                // collision::gather_contacts_aa(&[self.wall.body], &mut self.ww);

                // collision between player and wall
                collision::gather_contacts_ab(&pb, &self.wall.body, &mut self.pw);
                // collision between floor and wall
                collision::gather_contacts_ab(&self.wall.body, &[self.floor.body], &mut self.fw);

                // collision between player and floor
                collision::gather_contacts_ab(&pb, &[self.floor.body], &mut self.pf);

                // restitute between player and moving wall
                // collision::restitute_dyn_dyn(&mut pb, &mut pv, &[self.wall.body], &mut self.pw);

                // restitute between player and floor
                collision::restitute_dyn_stat(&mut pb, &mut pv, &[self.floor.body], &mut self.pf);

                // println!("wall - wall: {:?}", self.ww);
                // println!("player - wall: {:?}", self.pw);
                // println!("floor - wall: {:?}", self.fw);
                // println!("player - floor: {:?}", self.pf);

                // if player hits wall, end game
                if !self.pw.is_empty() {
                    // TODO: explode wall
                    // reset player position
                    self.player.body.c = Pos3::new(0.0, PBHS, 0.0);
                    self.mode = Mode::EndScreen
                } else {
                    self.player.body = pb[0];
                    self.player.velocity = pv[0];
                }
            }
            Mode::EndScreen => {
                self.ps.clear();

                // collision between player and play again menu object
                collision::gather_contacts_ab(&pb, &[self.play_again.body], &mut self.ps);

                // collision between player and floor
                collision::gather_contacts_ab(&pb, &[self.floor.body], &mut self.pf);

                // restitute between player and floor
                collision::restitute_dyn_stat(&mut pb, &mut pv, &[self.floor.body], &mut self.pf);

                // if player hits play again menu object, start game
                if !self.ps.is_empty() {
                    // reset player position
                    self.player.body.c = Pos3::new(0.0, PBHS, 0.0);
                    self.mode = Mode::GamePlay;
                } else {
                    self.player.body = pb[0];
                    self.player.velocity = pv[0];
                }
            }
        }
    }

    fn update(&mut self, _rules: &Self::StaticData, engine: &mut Engine) {
        // dbg!(self.player.body);
        // TODO update player acc with controls
        // TODO update camera with controls/player movement
        // TODO TODO show how spherecasting could work?  camera pseudo-entity collision check?  camera entity for real?
        // self.camera_controller.update(engine);

        self.player.acc = Vec3::zero();

        // how much the player moves per button click
        let h_disp = Vec3::new(0.05, 0.0, 0.0);
        let v_disp = Vec3::new(0.0, 0.15, 0.0);

        // player should not go past these bounds
        let top_bound = WH as f32 * WBHS * 2.0;
        let left_bound = WW as f32 * WBHS - WBHS;
        let right_bound = -left_bound;

        // move player
        let psn = self.player.body.c;
        if engine.events.key_held(KeyCode::A) && psn.x + PBHS + h_disp.x <= left_bound {
            // self.player.body.c += h_disp;
            self.player.velocity += h_disp;
        } else if engine.events.key_held(KeyCode::D) && psn.x + PBHS - h_disp.x >= right_bound {
            // self.player.body.c -= h_disp;
            self.player.velocity -= h_disp;
        } else if engine.events.key_held(KeyCode::Space) && psn.y + PBHS + v_disp.y <= top_bound {
            // self.player.body.c += v_disp;
            self.player.velocity += v_disp;
        } else {
            self.player.velocity = Vec3::zero();
        }

        if self.player.acc.magnitude2() > 1.0 {
            self.player.acc = self.player.acc.normalize();
        }

        // TODO: remove this?
        if engine.events.key_held(KeyCode::Q) {
            self.player.omega = Vec3::unit_y();
        } else if engine.events.key_held(KeyCode::E) {
            self.player.omega = -Vec3::unit_y();
        } else {
            self.player.omega = Vec3::zero();
        }

        // orbit camera
        self.camera.update(&engine.events, self.player.body.c);

        self.wall.integrate();
        self.floor.integrate();
        self.player.integrate();
        self.camera.integrate();

        self.handle_collision();

        for collision::Contact { a: pa, .. } in self.pf.iter() {
            // apply "friction" to players on the ground
            assert_eq!(*pa, 0);
            // self.player.velocity *= 0.98;
        }

        if !self.pw.is_empty() {
            // Explode wall
            for pos in 0..self.wall.body.len() {
                self.wall.vels[pos] += (self.wall.body[pos].c - self.player.body.c) * 2.0;
            }
        }

        self.camera.update_camera(engine.camera_mut());
    }
}

fn main() {
    env_logger::init();
    let title = env!("CARGO_PKG_NAME");
    let window = winit::window::WindowBuilder::new().with_title(title);
    run::<GameData, Game<OrbitCamera>>(window, std::path::Path::new("content"));
}
