use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rle<T> {
    pub runs: Vec<Run<T>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Run<T> {
    pub value: T,
    pub count: u32,
}

pub fn encode<T: Clone + PartialEq>(data: &[T]) -> Rle<T> {
    if data.is_empty() {
        return Rle { runs: Vec::new() };
    }

    let mut runs = Vec::new();
    let mut current_run = Run {
        value: data[0].clone(),
        count: 1,
    };
    for value in &data[1..] {
        if *value == current_run.value {
            current_run.count += 1;
        } else {
            runs.push(current_run);
            current_run = Run {
                value: value.clone(),
                count: 1,
            };
        }
    }
    runs.push(current_run);
    Rle { runs }
}

pub fn decode<T: Clone>(rle: &Rle<T>) -> Vec<T> {
    rle.runs
        .iter()
        .flat_map(|run| std::iter::repeat(run.value.clone()).take(run.count as usize))
        .collect()
}
