use num_traits::Float;

pub ASDRState {
    Attack,
    Sustain,
    Release,
    Finished
}

pub trait Envelope<T : Float> {
    next (&mut self) -> (),
    trigger (&mut self) -> (),
    release (&mut self) -> (),
    get_value () -> T,
    is_finished () -> bool,
}

pub struct ASDREnvelope<T : Float> {
    attack : f64,
    sustain: T,
    decay : f64,
    release: u64,
    state: ASDRState,
    triggerCounter: u64 // in samlples
    releaseCounter: u64
}

impl<T> ASDREnvelope<T : Float> {
    set_asdr (&mut self, attack: i64, sustain: T, decay: u64, release: u64) {
        self.attack = attack
        self.sustain = sustain
        self.decay = decay
        self.release = release
    }
}

impl<T> Envelope<T> for ASDREnvelope<T> {
    next (&mut self) {
        match self.state {
            Attack =>
        }
        }
        if self.releaseCounter > 0 self.releaseCounter--;
    }

    get_value () -> T {
    }
}
