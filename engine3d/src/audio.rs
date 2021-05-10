use ambisonic::{Ambisonic, AmbisonicBuilder, rodio, rodio::Source, SoundController};
use std::rc::Rc;

#[derive(PartialEq, Eq, Clone, Copy, Hash, Debug)]
pub struct SoundID(pub usize);

#[derive(PartialEq, Clone, Copy, Debug)]
pub enum AlreadyPlayingAction {
    Play,
    Retrigger,
    Nothing,
}

pub struct Audio {
    scene: Ambisonic,
    // sources: Vec<Rc<rodio::Decoder<std::io::BufReader<std::fs::File>>>>,
    source1: rodio::Decoder<std::io::BufReader<std::fs::File>>,
    source2: rodio::Decoder<std::io::BufReader<std::fs::File>>,
    source3: rodio::Decoder<std::io::BufReader<std::fs::File>>,
    source4: rodio::Decoder<std::io::BufReader<std::fs::File>>,
    playing: Vec<bool>,
    playing_action: Vec<AlreadyPlayingAction>,
    controller: Vec<Option<SoundController>>
}

impl Audio {
    pub fn new(paths: Vec<String>, repeats: Vec<bool>, playing_action: Vec<AlreadyPlayingAction>) -> Self {
        let scene = AmbisonicBuilder::default().build();
        let mut sources = vec![];
        let mut controller: Vec<Option<SoundController>> = vec![];
        for path in paths {
            let file = std::fs::File::open(path).unwrap();
            let source = rodio::Decoder::new(std::io::BufReader::new(file)).unwrap();
            sources.push(Rc::new(source));
            controller.push(None);
        }
        let source1 = rodio::Decoder::new(std::io::BufReader::new(file1)).unwrap();
        let source2 = rodio::Decoder::new(std::io::BufReader::new(file2)).unwrap();
        let source3 = rodio::Decoder::new(std::io::BufReader::new(file3)).unwrap();
        let source4 = rodio::Decoder::new(std::io::BufReader::new(file4)).unwrap();
        Self {
            scene,
            // sources,
            playing: vec![false; 4],
            playing_action,
            controller,
        }
    }

    pub fn play_at(
        &mut self,
        id: SoundID,
        posn: [f32; 3],
    ) {
        if !self.playing[id.0] {
            // if sound is not currently playing, play it 
            let source = &self.sources[id.0];
            let sound = self.scene.play_at(source.convert_samples(), posn);
            self.controller[id.0] = Some(sound);
            self.playing[id.0] = true;
        } else {
            // if the sound is currently playing, handle appropriately
            match self.playing_action[id.0] {
                AlreadyPlayingAction::Play => {
                    // play sound 
                    let sound = self.scene.play_at(self.sources[id.0].convert_samples(), posn);
                    self.controller[id.0] = Some(sound);
                    self.playing[id.0] = true;
                }
                AlreadyPlayingAction::Retrigger => {
                    self.stop(id);
                    let sound = self.scene.play_at(self.sources[id.0].convert_samples(), posn);
                    self.controller[id.0] = Some(sound);
                }
                AlreadyPlayingAction::Nothing => {
                    self.update_posn(id, posn);
                }
            }
        }
    }

    pub fn stop(&mut self, id: SoundID) {
        self.controller[id.0].unwrap().stop();
        self.playing[id.0] = false;
        self.controller[id.0] = None;
    }

    pub fn update_posn(&mut self, id: SoundID, posn: [f32; 3]) {
        self.controller[id.0].unwrap().adjust_position(posn);
    }
}
