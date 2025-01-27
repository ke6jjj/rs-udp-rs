use ndarray::Array1;
use num_traits::Float;
use std::path::Path;
use std::{fmt::Display, fs::File, io::Write};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ObserverError {
    #[error("unable to open dump file")]
    DumpFileError(#[from] std::io::Error),
}

#[derive(Debug, Clone, Copy)]
pub enum FilterStep {
    Input,
    Affined,
    Filtered,
    DCRemove,
    Energy,
}

pub enum FilterObserver<T> {
    NullObserver,
    ChannelDumper(Box<ChannelDumper<T>>),
}

impl<T: Float + Display> FilterObserver<T> {
    pub fn new_channel_dumper(path: &Path) -> Result<FilterObserver<T>, ObserverError> {
        let c = ChannelDumper::new(path)?;
        Ok(FilterObserver::ChannelDumper(Box::new(c)))
    }

    pub fn null() -> Result<FilterObserver<T>, ObserverError> {
        Ok(FilterObserver::NullObserver)
    }

    pub fn observe(&mut self, step: FilterStep, n: usize, input: &ndarray::Array1<T>) {
        match self {
            Self::NullObserver => (),
            Self::ChannelDumper(d) => d.observe(step, n, input),
        }
    }
}

pub struct ChannelDumper<T> {
    f: File,
    input: ndarray::Array1<T>,
    affine: ndarray::Array1<T>,
    filtered: ndarray::Array1<T>,
    dc_removed: ndarray::Array1<T>,
    energy: ndarray::Array1<T>,
}

impl<T: Float> ChannelDumper<T> {
    pub fn new(path: &Path) -> Result<ChannelDumper<T>, ObserverError> {
        let fh = std::fs::File::create(path)?;
        Ok(ChannelDumper {
            f: fh,
            input: Array1::<T>::from_vec(vec![]),
            affine: Array1::<T>::from_vec(vec![]),
            filtered: Array1::<T>::from_vec(vec![]),
            dc_removed: Array1::<T>::from_vec(vec![]),
            energy: Array1::<T>::from_vec(vec![]),
        })
    }

    fn check_all_received(&self, n: usize) {
        if self.input.len() != n
            || self.affine.len() != n
            || self.filtered.len() != n
            || self.dc_removed.len() != n
            || self.energy.len() != n
        {
            panic!("observed array lengths unequal")
        }
    }
}

impl<T: Float + Display> ChannelDumper<T> {
    fn observe(&mut self, step: FilterStep, n: usize, input: &ndarray::Array1<T>) {
        match step {
            FilterStep::Input => self.input = input.clone(),
            FilterStep::Affined => self.affine = input.clone(),
            FilterStep::Filtered => self.filtered = input.clone(),
            FilterStep::DCRemove => self.dc_removed = input.clone(),
            FilterStep::Energy => {
                self.energy = input.clone();
                self.check_all_received(self.energy.len());

                for i in 0..self.input.len() {
                    let off: f32 = ((n + i) as f32) / 100.0;
                    let inp = self.input[i];
                    let aff = self.affine[i];
                    let fil = self.filtered[i];
                    let dc = self.dc_removed[i];
                    let energy = self.energy[i];
                    writeln!(self.f, "{off} {inp} {aff} {fil} {dc} {energy}")
                        .expect("can't dump to file");
                }
            }
        }
    }
}
