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

const G: f32 = 5.0;
const MIN_VEL: f32 = 0.1; // if absolute velocity is below this value, consider the object to be stationary
const MBHS: f32 = 0.5; // menu box half size
const WBHS: f32 = 1.0; // wall box half size
const PBHS: f32 = 0.5; // player box half size
const WH: i8 = 3; // wall height in boxes
const WW: i8 = 6; // wall width in boxes
const WIV: Vec3 = Vec3::new(0.0, 0.0, -2.0); // initial velocity of wall
const WIZ: f32 = 20.0; // initial z position of wall
const WVSF: f32 = 0.5; // wall velocity scaling factor

#[derive(Clone, PartialEq, Debug)]
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
    pub rots: Vec<Quat>,
    pub omegas: Vec<Vec3>,
    control: (i8, i8),
}

impl Wall {
    pub fn generate_components(axes: Mat3) -> Vec<Box> {
        let mut rng = rand::thread_rng();
        let missing_x = rng.gen_range(0..WW);
        let missing_y = rng.gen_range(0..WH);

        let mut boxes = vec![];
        let half_sizes = Vec3::new(WBHS, WBHS, WBHS);
        for x in 0..WW {
            for y in 0..WH {
                if x != missing_x || y != missing_y {
                    let c = Pos3::new(
                        x as f32 * 2.0 * WBHS + WBHS - WW as f32 * WBHS,
                        y as f32 * 2.0 * WBHS + WBHS,
                        WIZ,
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

    fn reset(&mut self, score: i8) {
        self.body = Wall::generate_components(Mat3::one());
        let n_boxes = self.body.len();
        self.vels = vec![WIV * (score + 1) as f32 * WVSF; n_boxes];
        self.rots = vec![Quat::new(1.0, 0.0, 0.0, 0.0); n_boxes];
        self.omegas = vec![Vec3::zero(); n_boxes];
        self.control = (0, 0);
    }

    fn render(&self, rules: &GameData, igs: &mut InstanceGroups) {
        for (i, b) in self.body.iter().enumerate() {
            igs.render(
                rules.wall_model,
                InstanceRaw {
                    model: (Mat4::from_translation(b.c.to_vec())
                        * Mat4::from_nonuniform_scale(
                            b.half_sizes.x,
                            b.half_sizes.y,
                            b.half_sizes.z,
                        )
                        * Mat4::from(self.rots[i]))
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

        for i in 0..self.body.len() {
            let drot = 0.5
                * DT
                * Quat::new(0.0, self.omegas[i].x, self.omegas[i].y, self.omegas[i].z)
                * self.rots[i];
            self.rots[i] += drot;
            self.body[i].axes = self.body[i].axes * Matrix3::from(drot);
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
    ps: Vec<collision::Contact<usize>>,
    ww: Vec<collision::Contact<usize>>,
    pw: Vec<collision::Contact<usize>>,
    fw: Vec<collision::Contact<usize>>,
    pf: Vec<collision::Contact<usize>>,
    mode: Mode,
    score: i8,
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
                model: (Mat4::from_translation(self.body.c.to_vec())
                    * Mat4::from_nonuniform_scale(
                        self.body.half_sizes.x,
                        self.body.half_sizes.y,
                        self.body.half_sizes.z,
                    )
                    * Mat4::from(self.rot))
                .into(),
            },
        );
    }
    fn integrate(&mut self) {
        self.velocity += self.rot * self.acc;
        // println!("inte {:?}", self.velocity);
        if self.velocity.magnitude() > Self::MAX_SPEED {
            self.velocity = self.velocity.normalize_to(Self::MAX_SPEED);
        }
        if self.velocity.magnitude() >= MIN_VEL {
            self.body.c += self.velocity * DT;
        }
        let drot = 0.5 * DT * Quat::new(0.0, self.omega.x, self.omega.y, self.omega.z) * self.rot;
        self.rot += drot;
        self.body.axes = self.body.axes * Matrix3::from(drot);
    }
}

impl<C: Camera> engine3d::Game for Game<C> {
    type StaticData = GameData;
    fn start(engine: &mut Engine) -> (Self, Self::StaticData) {
        use rand::Rng;

        // create menu objects
        let menu_object_half_sizes = Vec3::new(MBHS, MBHS, MBHS);
        let start = MenuObject {
            body: Box {
                c: Pos3::new(3.0, MBHS, 0.0),
                axes: Matrix3::one(),
                half_sizes: menu_object_half_sizes,
            },
        };
        let scores = MenuObject {
            body: Box {
                c: Pos3::new(-3.0, MBHS, 0.0),
                axes: Matrix3::one(),
                half_sizes: menu_object_half_sizes,
            },
        };
        let play_again = MenuObject {
            body: Box {
                c: Pos3::new(3.0, MBHS, 0.0),
                axes: Matrix3::one(),
                half_sizes: menu_object_half_sizes,
            },
        };

        // create wall
        // generate wall components
        // let boxes = Wall::generate_components(Matrix3::one());
        let boxes = Wall::generate_components(Matrix3::one());
        let n_boxes = boxes.len();
        let wall = Wall {
            body: boxes,
            vels: vec![WIV; n_boxes],
            rots: vec![Quat::new(1.0, 0.0, 0.0, 0.0); n_boxes],
            omegas: vec![Vec3::zero(); n_boxes],
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
                ps: vec![],
                ww: vec![],
                fw: vec![],
                pw: vec![],
                pf: vec![],
                mode: Mode::Menu,
                score: 0,
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
                self.wall.render(rules, igs);
                self.play_again.render(rules, igs);
                self.scores.render(rules, igs);
            }
        }
    }

    fn handle_collision(&mut self) {
        self.pf.clear();
        self.pw.clear();
        let mut pb = [self.player.body];
        let mut pv = [self.player.velocity];

        // always check and restitute player - floor
        collision::gather_contacts_ab(&pb, &[self.floor.body], &mut self.pf);
        collision::restitute_dyn_stat(&mut pb, &mut pv, &[self.floor.body], &mut self.pf, false);
        // always check and restitute player - wall
        collision::gather_contacts_ab(&pb, &self.wall.body, &mut self.pw);
        collision::restitute_dyn_dyn(
            &mut pb,
            &mut pv,
            &mut self.wall.body,
            &mut self.wall.vels,
            &mut self.pw,
        );

        match self.mode {
            Mode::Menu => {
                self.ps.clear();
                // collision between player and start object
                collision::gather_contacts_ab(&pb, &[self.start.body], &mut self.ps);
            }
            Mode::GamePlay => {
                self.ww.clear();
                self.fw.clear();

                // wall - wall
                collision::gather_contacts_aa(&self.wall.body, &mut self.ww);
                // collision::restitute_dyns(&mut self.wall.body, &mut self.wall.vels, &mut self.ww);

                // println!("wall - wall: {:?}", self.ww);
                // println!("player - wall: {:?}", self.pw);
                // println!("floor - wall: {:?}", self.fw);
                // println!("player - floor: {:?}", self.pf);
            }
            Mode::EndScreen => {
                self.ps.clear();
                self.ww.clear();
                self.fw.clear();

                // collision between player and play again menu object
                collision::gather_contacts_ab(&pb, &[self.play_again.body], &mut self.ps);

                // wall - wall
                collision::gather_contacts_aa(&self.wall.body, &mut self.ww);
                collision::restitute_dyns(&mut self.wall.body, &mut self.wall.vels, &mut self.ww);

                // floor - wall
                collision::gather_contacts_ab(&self.wall.body, &[self.floor.body], &mut self.fw);
                collision::restitute_dyn_stat(
                    &mut self.wall.body,
                    &mut self.wall.vels,
                    &[self.floor.body],
                    &mut self.fw,
                    true,
                );
            }
        }
        self.player.body = pb[0];
        self.player.velocity = pv[0];
        // self.player.body.c += self.player.velocity * DT;
    }

    fn update(&mut self, _rules: &Self::StaticData, engine: &mut Engine) {
        // dbg!(self.player.body);
        // TODO update player acc with controls
        // TODO update camera with controls/player movement
        // TODO TODO show how spherecasting could work?  camera pseudo-entity collision check?  camera entity for real?
        // self.camera_controller.update(engine);

        self.player.acc = Vec3::zero();

        // how much the player velocity changes per button click
        let h_disp = Vec3::new(0.05, 0.0, 0.0);
        let v_disp = Vec3::new(0.0, 0.30, 0.0);
        let g_disp = Vec3::new(0.0, -G, 0.0);

        // player should not go past these bounds
        let top_bound = WH as f32 * WBHS * 2.0;
        let left_bound = WW as f32 * WBHS - 2.0;
        let right_bound = -left_bound + WBHS;

        // apply gravity here instead of integrate() so handle_collision can deal with gravity smoothly
        self.player.velocity += g_disp * DT;
        if self.mode == Mode::EndScreen {
            for v in self.wall.vels.iter_mut() {
                *v += g_disp * DT;
            }
        }

        self.handle_collision();

        // move player
        let psn = self.player.body.c;
        if engine.events.key_held(KeyCode::A) && psn.x + PBHS + h_disp.x <= left_bound {
            // self.player.body.c += h_disp;
            self.player.acc += h_disp;
        } else if engine.events.key_held(KeyCode::D) && psn.x + PBHS - h_disp.x >= right_bound {
            // self.player.body.c -= h_disp;
            self.player.acc -= h_disp;
        } else if engine.events.key_held(KeyCode::Space) && psn.y + PBHS + v_disp.y <= top_bound {
            // self.player.body.c += v_disp;
            self.player.acc += v_disp;
        }

        if self.player.acc.magnitude2() > 1.0 {
            self.player.acc = self.player.acc.normalize();
        }

        // rotate player
        if engine.events.key_held(KeyCode::Q) {
            self.player.omega = Vec3::unit_y();
        } else if engine.events.key_held(KeyCode::E) {
            self.player.omega = -Vec3::unit_y();
        } else {
            self.player.omega = Vec3::zero();
        }

        // orbit camera
        self.camera.update(&engine.events, self.player.body.c);

        if self.mode != Mode::Menu {
            self.wall.integrate();
        }
        self.floor.integrate();
        self.player.integrate();
        self.camera.integrate();

        for collision::Contact { a: pa, .. } in self.pf.iter() {
            // apply "friction" to players on the ground
            assert_eq!(*pa, 0);
            self.player.velocity *= 0.98;
        }

        match self.mode {
            Mode::Menu => {
                // if player hits start menu object, start game
                if !self.ps.is_empty() {
                    self.mode = Mode::GamePlay;
                    // reset player position
                    self.player.body.c = Pos3::new(0.0, PBHS, 0.0);
                }
            }
            Mode::GamePlay => {
                // if player hits wall, end game
                if !self.pw.is_empty() {
                    self.mode = Mode::EndScreen;
                    // Explode wall, away from player and toward the back
                    for pos in 0..self.wall.body.len() {
                        // self.wall.vels[pos] +=
                            // (self.wall.body[pos].c - self.player.body.c - WIV * 3.0)
                                // .normalize_to(rand::random::<f32>());

                        self.wall.omegas[pos] = Vec3::new(
                            rand::random::<f32>(),
                            rand::random::<f32>(),
                            rand::random::<f32>(),
                        )
                        .normalize();
                    }
                    // TODO: record and write score to file
                    // reset score and player position
                    self.score = 0;
                    self.player.body.c = Pos3::new(0.0, PBHS, 0.0);
                } else if self.wall.body[0].c.z + WBHS < self.player.body.c.z - 2.0 * WBHS {
                    // if wall passes camera, increment score and reset wall
                    self.score += 1;
                    self.wall.reset(self.score);
                }
            }
            Mode::EndScreen => {
                // if player hits play again menu object, start game
                if !self.ps.is_empty() {
                    self.mode = Mode::GamePlay;
                    // reset wall and player position
                    self.player.body.c = Pos3::new(0.0, PBHS, 0.0);
                    self.wall.reset(self.score);
                }

                // clear wall blocks from view once they get far away
                let mut to_keep: Vec<bool> = Vec::new();
                for i in 0..self.wall.body.len() {
                    if (self.wall.body[i].c - self.player.body.c).magnitude() < 50.0 {
                        to_keep.push(true);
                    } else {
                        to_keep.push(false);
                    }
                }
                self.wall.body.retain(|_| *to_keep.iter().next().unwrap());
                self.wall.vels.retain(|_| *to_keep.iter().next().unwrap());
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
