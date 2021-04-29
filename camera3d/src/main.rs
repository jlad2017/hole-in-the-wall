use engine3d::{collision, events::*, geom::*, render::InstanceGroups, run, Engine, DT};
use rand;
use winit;
use winit::event::VirtualKeyCode as KeyCode;

const NUM_MARBLES: usize = 10;
const G: f32 = 1.0;

#[derive(Clone, Debug)]
pub struct Player {
    pub body: Sphere,
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
            engine3d::render::InstanceRaw {
                model: (Mat4::from_translation(self.body.c.to_vec() - Vec3::new(0.0, 0.2, 0.0))
                    * Mat4::from_scale(self.body.r)
                    * Mat4::from(self.rot))
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

trait Camera {
    fn new() -> Self;
    fn update(&mut self, _events: &engine3d::events::Events, _player: &Player) {}
    fn render(&self, _rules: &GameData, _igs: &mut InstanceGroups) {}
    fn update_camera(&self, _cam: &mut engine3d::camera::Camera) {}
    fn integrate(&mut self) {}
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Wall {
    pub body: Plane,
    control: (i8, i8),
}

impl Wall {
    fn render(&self, rules: &GameData, igs: &mut InstanceGroups) {
        igs.render(
            rules.wall_model,
            engine3d::render::InstanceRaw {
                model: (Mat4::from(cgmath::Quaternion::between_vectors(
                    Vec3::new(0.0, 1.0, 0.0),
                    self.body.n,
                )) * Mat4::from_translation(Vec3::new(0.0, -0.025, 0.0))
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
    platform: Wall,
    player: Player,
    camera: Cam,
    pm: Vec<collision::Contact<usize>>,
    pw: Vec<collision::Contact<usize>>,
    mm: Vec<collision::Contact<usize>>,
    mw: Vec<collision::Contact<usize>>,
}
struct GameData {
    marble_model: engine3d::assets::ModelRef,
    wall_model: engine3d::assets::ModelRef,
    player_model: engine3d::assets::ModelRef,
    camera_model: engine3d::assets::ModelRef,
}

impl<C: Camera> engine3d::Game for Game<C> {
    type StaticData = GameData;
    fn start(engine: &mut Engine) -> (Self, Self::StaticData) {
        use rand::Rng;
        let wall = Wall {
            body: Plane {
                n: Vec3::new(0.0, 1.0, 0.0),
                d: 0.0,
            },
            control: (0, 0),
        };
        let player = Player {
            body: Sphere {
                c: Pos3::new(0.0, 3.0, 0.0),
                r: 0.3,
            },
            velocity: Vec3::zero(),
            acc: Vec3::zero(),
            omega: Vec3::zero(),
            rot: Quat::new(1.0, 0.0, 0.0, 0.0),
        };
        let camera = C::new();
        let mut rng = rand::thread_rng();
        let marbles = Marbles {
            body: (0..NUM_MARBLES)
                .map(move |_x| {
                    let x = rng.gen_range(-5.0..5.0);
                    let y = rng.gen_range(1.0..5.0);
                    let z = rng.gen_range(-5.0..5.0);
                    let r = rng.gen_range(0.1..1.0);
                    Sphere {
                        c: Pos3::new(x, y, z),
                        r,
                    }
                })
                .collect::<Vec<_>>(),
            velocity: vec![Vec3::zero(); NUM_MARBLES],
        };
        let wall_model = engine.load_model("floor.obj");
        let marble_model = engine.load_model("sphere.obj");
        let player_model = engine.load_model("capsule.obj");
        let camera_model = engine.load_model("sphere.obj");
        (
            Self {
                // camera_controller,
                marbles,
                wall,
                player,
                camera,
                // TODO nice this up somehow
                mm: vec![],
                mw: vec![],
                pm: vec![],
                pw: vec![],
            },
            GameData {
                wall_model,
                marble_model,
                player_model,
                camera_model,
            },
        )
    }
    fn render(&self, rules: &Self::StaticData, igs: &mut InstanceGroups) {
        self.wall.render(rules, igs);
        self.marbles.render(rules, igs);
        self.player.render(rules, igs);
        // self.camera.render(rules, igs);
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
        self.camera.update(&engine.events, &self.player);

        self.wall.integrate();
        self.player.integrate();
        self.marbles.integrate();
        self.camera.integrate();

        {
            use rand::Rng;
            let mut rng = rand::thread_rng();
            for (body, vel) in self.marbles.iter_mut() {
                if (body.c.distance(Pos3::new(0.0, 0.0, 0.0))) >= 40.0 {
                    body.c = Pos3::new(
                        rng.gen_range(-5.0..5.0),
                        rng.gen_range(1.0..5.0),
                        rng.gen_range(-5.0..5.0),
                    );
                    *vel = Vec3::zero();
                }
            }
        }
        self.mm.clear();
        self.mw.clear();
        self.pm.clear();
        self.pw.clear();
        let mut pb = [self.player.body];
        let mut pv = [self.player.velocity];
        collision::gather_contacts_ab(&pb, &self.marbles.body, &mut self.pm);
        collision::gather_contacts_ab(&pb, &[self.wall.body], &mut self.pw);
        collision::gather_contacts_ab(&self.marbles.body, &[self.wall.body], &mut self.mw);
        collision::gather_contacts_aa(&self.marbles.body, &mut self.mm);
        collision::restitute_dyn_stat(&mut pb, &mut pv, &[self.wall.body], &mut self.pw);
        collision::restitute_dyn_stat(
            &mut self.marbles.body,
            &mut self.marbles.velocity,
            &[self.wall.body],
            &mut self.mw,
        );
        collision::restitute_dyns(
            &mut self.marbles.body,
            &mut self.marbles.velocity,
            &mut self.mm,
        );
        collision::restitute_dyn_dyn(
            &mut pb,
            &mut pv,
            &mut self.marbles.body,
            &mut self.marbles.velocity,
            &mut self.pm,
        );
        self.player.body = pb[0];
        self.player.velocity = pv[0];

        for collision::Contact { a: ma, .. } in self.mw.iter() {
            // apply "friction" to marbles on the ground
            self.marbles.velocity[*ma] *= 0.995;
        }
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
    // run::<GameData, Game<OrbitCamera>>(window, std::path::Path::new("content"));
    run::<GameData, Game<LOLCamera>>(window, std::path::Path::new("content"));
}
