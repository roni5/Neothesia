use std::time::Duration;

use {
    crate::TracksParser,
    midly::{MetaMessage, MidiMessage, TrackEvent, TrackEventKind},
    std::collections::HashMap,
};

#[derive(Debug, Clone)]
pub struct TempoEvent {
    pub time_in_units: u64,
    pub tempo: u32,
}

#[derive(Debug, Clone)]
pub struct MidiNote {
    pub start: Duration,
    pub duration: Duration,
    pub note: u8,
    pub vel: u8,
    pub ch: u8,
    pub track_id: usize,
    pub id: usize,
}

#[derive(Debug, Clone)]
pub struct MidiTrack {
    pub tempo: u32,
    pub tempo_events: Vec<TempoEvent>,
    pub has_tempo: bool,
    pub notes: Vec<MidiNote>,
    pub track_id: usize,
}

impl MidiTrack {
    pub fn new(track: &[TrackEvent], track_id: usize) -> Self {
        let mut tempo = 500_000; // 120 bpm

        let mut has_tempo = false;
        let mut tempo_events = Vec::new();

        let mut time_in_units: u64 = 0;
        for event in track.iter() {
            time_in_units += event.delta.as_int() as u64;

            if let TrackEventKind::Meta(meta) = &event.kind {
                if let MetaMessage::Tempo(t) = &meta {
                    if !has_tempo {
                        tempo = t.as_int();
                        has_tempo = true;
                    }
                    tempo_events.push(TempoEvent {
                        time_in_units,
                        tempo: t.as_int(),
                    });
                }
            };
        }

        Self {
            tempo,
            tempo_events,
            has_tempo,
            track_id,
            notes: Vec::new(),
        }
    }

    pub fn extract_notes(&mut self, events: &[TrackEvent], parent_parser: &mut TracksParser) {
        self.notes.clear();

        let mut time_in_units = 0u64;

        struct Note {
            time_in_units: u64,
            vel: u8,
            channel: u8,
        }
        let mut current_notes: HashMap<u8, Note> = HashMap::new();

        macro_rules! end_note {
            (k => $e:expr) => {
                let k = $e;
                if current_notes.contains_key(&k) {
                    let n = current_notes.get(&k).unwrap();

                    let start = parent_parser.pulses_to_micro(n.time_in_units);
                    let duration = parent_parser.pulses_to_micro(time_in_units) - start;

                    let mn = MidiNote {
                        start,
                        duration,
                        note: k,
                        vel: n.vel,
                        ch: n.channel,
                        track_id: self.track_id,
                        id: 0, // Placeholder
                    };
                    self.notes.push(mn);
                    current_notes.remove(&k);
                }
            };
        }

        for event in events.iter() {
            time_in_units += event.delta.as_int() as u64;

            if let TrackEventKind::Midi { channel, message } = &event.kind {
                match &message {
                    MidiMessage::NoteOn { key, vel } => {
                        let key = key.as_int();
                        let vel = vel.as_int();

                        match vel.cmp(&0) {
                            std::cmp::Ordering::Greater => {
                                let k = key;

                                match current_notes.entry(k) {
                                    std::collections::hash_map::Entry::Occupied(_e) => {
                                        end_note!(k=>k);
                                    }
                                    std::collections::hash_map::Entry::Vacant(_e) => {
                                        current_notes.insert(
                                            k,
                                            Note {
                                                time_in_units,
                                                vel,
                                                channel: channel.as_int(),
                                            },
                                        );
                                    }
                                }
                            }
                            std::cmp::Ordering::Equal => {
                                end_note!(k=>key);
                            }
                            _ => {}
                        }
                    }
                    MidiMessage::NoteOff { key, .. } => {
                        let key = key.as_int();

                        end_note!(k=>key);
                    }
                    _ => {}
                }
            }
        }

        self.notes
            .sort_by(|a, b| a.start.partial_cmp(&b.start).unwrap());
    }
}
