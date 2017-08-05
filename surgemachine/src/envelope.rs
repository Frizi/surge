enum ADSRState {
    Idle,
    Attack,
    Decay,
    Sustain,
    Release,
}

pub trait Envelope {
    fn process (&mut self) -> ();
    fn trigger (&mut self) -> ();
    fn release (&mut self) -> ();
    fn get_value (&self) -> f32;
    fn is_finished (&self) -> bool;
    fn reset (&mut self) -> ();
}

pub struct ADSREnvelope {
    state: ADSRState,
    output: f32,
    attack_rate: f32,
    decay_rate: f32,
    release_rate: f32,
    attack_coef: f32,
    decay_coef: f32,
    release_coef: f32,
    sustain_level: f32,
    target_ratio_a: f32,
    target_ratio_dr: f32,
    attack_base: f32,
    decay_base: f32,
    release_base: f32,
}

impl ADSREnvelope {
    pub fn set_adsr (&mut self, sample_rate: f32, attack: f32, decay: f32, sustain: f32, release: f32) {
        self.target_ratio_a = 0.3;
        self.target_ratio_dr = 0.0001;
        self.sustain_level = sustain.min(1.0).max(0.0);

        self.attack_rate = attack * sample_rate;
        self.attack_coef = calc_coef(self.attack_rate, self.target_ratio_a);
        self.attack_base = (1.0 + self.target_ratio_a) * (1.0 - self.attack_coef);

        self.decay_rate = decay * sample_rate;
        self.decay_coef = calc_coef(self.decay_rate, self.target_ratio_dr);
        self.decay_base = (self.sustain_level - self.target_ratio_dr) * (1.0 - self.decay_coef);

        self.release_rate = release * sample_rate;
        self.release_coef = calc_coef(self.release_rate, self.target_ratio_dr);
        self.release_base = -self.target_ratio_dr * (1.0 - self.release_coef);
    }
}

impl Default for ADSREnvelope {
    fn default () -> ADSREnvelope {
        ADSREnvelope {
            state: ADSRState::Idle,
            output: 0.0,
            attack_rate: 0.0,
            decay_rate: 0.0,
            release_rate: 0.0,
            attack_coef: 0.0,
            decay_coef: 0.0,
            release_coef: 0.0,
            sustain_level: 0.0,
            target_ratio_a: 0.3,
            target_ratio_dr: 0.0001,
            attack_base: 0.0,
            decay_base: 0.0,
            release_base: 0.0,
        }
    }
}

impl Envelope for ADSREnvelope {
    fn process (&mut self) {
        match self.state {
            ADSRState::Attack => {
                self.output = self.attack_base + self.output * self.attack_coef;
                if self.output >= 1.0 {
                    self.output = 1.0;
                    self.state = ADSRState::Decay;
                }
            },
            ADSRState::Decay => {
                self.output = self.decay_base + self.output * self.decay_coef;
                if self.output <= self.sustain_level {
                    self.output = self.sustain_level;
                    self.state = ADSRState::Sustain;
                }
            },
            ADSRState::Release => {
                self.output = self.release_base + self.output * self.release_coef;
                if self.output <= 0.0 {
                    self.output = 0.0;
                    self.state = ADSRState::Idle;
                }
            }
            _ => ()
        }
    }

    fn trigger (&mut self) {
        self.state = ADSRState::Attack;
    }

    fn release (&mut self) {
        self.state = ADSRState::Release;
    }

    fn is_finished (&self) -> bool {
        match self.state {
            ADSRState::Idle => true,
            _ => false
        }
    }

    fn reset (&mut self) {
        self.state = ADSRState::Idle;
        self.output = 0.0;
    }

    #[inline]
    fn get_value (&self) -> f32 {
        self.output
    }
}

fn calc_coef(rate: f32, target_ratio: f32) -> f32 {
    if rate <= 0.0 { 0.0 }
    else { (-((1.0 + target_ratio) / target_ratio).ln() / rate).exp() }
}
