#[derive(PartialEq)]
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
    fn get_value (&self) -> f64;
    fn is_finished (&self) -> bool;
    fn reset (&mut self) -> ();
}

pub struct ADSREnvelope {
    state: ADSRState,
    output: f64,
    attack_rate: f64,
    decay_rate: f64,
    release_rate: f64,
    attack_coef: f64,
    decay_coef: f64,
    release_coef: f64,
    sustain_level: f64,
    target_ratio_a: f64,
    target_ratio_dr: f64,
    attack_base: f64,
    decay_base: f64,
    release_base: f64,
}

impl ADSREnvelope {
    #[allow(dead_code)]
    pub fn set_adsr (&mut self, sample_rate: f64, attack: f64, decay: f64, sustain: f64, release: f64) {
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

    fn is_finished (&self) -> bool { self.state == ADSRState::Idle }

    fn reset (&mut self) {
        self.state = ADSRState::Idle;
        self.output = 0.0;
    }

    fn get_value (&self) -> f64 {
        self.output
    }
}

fn calc_coef(rate: f64, target_ratio: f64) -> f64 {
    if rate <= 0.0 { 0.0 }
    else { (-((1.0 + target_ratio) / target_ratio).ln() / rate).exp() }
}
