use cgmath::Matrix3;
use engine3d::{
    camera::*,
    collision,
    geom::*,
    render::{InstanceGroups, InstanceRaw},
    run, Engine, DT,
};
use rand;
use winit;
use winit::event::VirtualKeyCode as KeyCode;

const G: f32 = 1.0;

#[derive(Clone, PartialEq, Debug)]
pub struct Wall {
    pub body: Vec<Box>,
    pub velocity: Vec3,
    control: (i8, i8),
}

impl Wall {
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
        for b in &mut self.body {
            b.c += self.velocity * DT;
        }
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Platform {
    pub body: Plane,
    control: (i8, i8),
}

impl Platform {
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
    wall: Wall,
    platform: Platform,
    player: Player,
    camera: Cam,
    pw: Vec<collision::Contact<usize>>,
    fw: Vec<collision::Contact<usize>>,
    pf: Vec<collision::Contact<usize>>,
}
struct GameData {
    wall_model: engine3d::assets::ModelRef,
    platform_model: engine3d::assets::ModelRef,
    player_model: engine3d::assets::ModelRef,
    camera_model: engine3d::assets::ModelRef,
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
                    // * Mat4::from_nonuniform_scale(
                    //     self.body.half_sizes.x,
                    //     self.body.half_sizes.y,
                    //     self.body.half_sizes.z,
                    // )
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
        let b1 = Box {
            c: Pos3::new(0.0, 0.0, 0.0),
            axes,
            half_sizes: Vec3::new(1.0, 1.0, 1.0),
        };
        let wall = Wall {
            body: vec![b1],
            velocity: Vec3::new(0.0, 0.0, 0.0),
            control: (0, 0),
        };
        let platform = Platform {
            body: Plane {
                n: Vec3::new(0.0, 1.0, 0.0),
                d: 0.0,
            },
            control: (0, 0),
        };
        let player = Player {
            body: Box {
                c: Pos3::new(0.0, 2.0, 0.0),
                axes: Matrix3::new(0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0),
                half_sizes: Vec3::new(1.0, 1.0, 1.0),
            },
            velocity: Vec3::zero(),
            acc: Vec3::zero(),
            omega: Vec3::zero(),
            rot: Quat::new(1.0, 0.0, 0.0, 0.0),
        };
        let camera = C::new();
        let mut rng = rand::thread_rng();
        let wall_model = engine.load_model("wall.obj");
        let platform_model = engine.load_model("floor.obj");
        let player_model = engine.load_model("cube.obj");
        let camera_model = engine.load_model("sphere.obj");
        (
            Self {
                wall,
                platform,
                player,
                camera,
                // TODO nice this up somehow
                pm: vec![],
                pw: vec![],
            },
            GameData {
                wall_model,
                platform_model,
                player_model,
                camera_model,
            },
        )
    }
    fn render(&self, rules: &Self::StaticData, igs: &mut InstanceGroups) {
        self.wall.render(rules, igs);
        self.platform.render(rules, igs);
        self.player.render(rules, igs);
    }
    fn update(&mut self, _rules: &Self::StaticData, engine: &mut Engine) {
        // dbg!(self.player.body);
        // TODO update player acc with controls
        // TODO update camera with controls/player movement
        // TODO TODO show how spherecasting could work?  camera pseudo-entity collision check?  camera entity for real?
        // self.camera_controller.update(engine);

        self.player.acc = Vec3::zero();
        if engine.events.key_held(KeyCode::W) {
            self.player.acc.z = 1.0;
        } else if engine.events.key_held(KeyCode::S) {
            self.player.acc.z = -1.0;
        }

        if engine.events.key_held(KeyCode::A) {
            self.player.acc.x = 1.0;
        } else if engine.events.key_held(KeyCode::D) {
            self.player.acc.x = -1.0;
        }
        if self.player.acc.magnitude2() > 1.0 {
            self.player.acc = self.player.acc.normalize();
        }

        if engine.events.key_held(KeyCode::Q) {
            self.player.omega = Vec3::unit_y();
        } else if engine.events.key_held(KeyCode::E) {
            self.player.omega = -Vec3::unit_y();
        } else {
            self.player.omega = Vec3::zero();
        }

        // orbit camera
        self.camera.update(&engine.events);

        self.wall.integrate();
        self.platform.integrate();
        self.player.integrate();
        self.camera.integrate();

        self.ww.clear();
        self.pw.clear();
        self.fw.clear();
        self.pf.clear();
        let mut pb = [self.player.body];
        let mut pv = [self.player.velocity];

        // collision between wall and wall
        collision::gather_contacts_aa(&[self.wall.body], &mut self.ww);

        for b in &self.wall.body {
            // collision between player and wall
            collision::gather_contacts_ab(&pb, &[*b], &mut self.pw);
            // collision between floor and wall
            collision::gather_contacts_ab(&[*b], &[self.platform.body], &mut self.fw);
        }

        // collision between player and floor
        collision::gather_contacts_ab(&pb, &[self.platform.body], &mut self.pf);

        // restitute between player and moving wall
        // collision::restitute_dyn_stat(&mut pb, &mut pv, &[self.wall.body], &mut self.pw);

        // restitute between player and platform
        collision::restitute_dyn_stat(&mut pb, &mut pv, &[self.platform.body], &mut self.pf);

        println!("wall - wall: {:?}", self.ww);
        println!("player - wall: {:?}", self.pw);
        println!("floor - wall: {:?}", self.fw);
        println!("player - floor: {:?}", self.pf);

        self.player.body = pb[0];
        self.player.velocity = pv[0];

        for collision::Contact { a: pa, .. } in self.pw.iter() {
            // apply "friction" to players on the ground
            assert_eq!(*pa, 0);
            self.player.velocity *= 0.98;
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
