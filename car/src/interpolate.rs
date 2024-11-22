#[derive(Debug, Clone)]
pub struct Interpolator1D {
    x: Vec<f64>,
    y: Vec<f64>,
}

impl Interpolator1D {
    pub fn new(x: Vec<f64>, y: Vec<f64>) -> Self {
        assert_eq!(x.len(), y.len());
        Self { x, y }
    }

    pub fn interpolate(&self, x: f64) -> f64 {
        // clamp x to the range of x
        if x <= self.x[0] {
            return self.y[0];
        }

        if x > self.x[self.x.len() - 1] {
            return self.y[self.y.len() - 1];
        }

        let i = bin_search(&self.x, x) - 1;

        // interpolate between the two points
        let x0 = self.x[i];
        let x1 = self.x[i + 1];
        let y0 = self.y[i];
        let y1 = self.y[i + 1];
        let slope = (y1 - y0) / (x1 - x0);
        y0 + slope * (x - x0)
    }
}

fn bin_search(x: &Vec<f64>, target: f64) -> usize {
    let mut i = 0;
    let mut j = x.len() - 1;
    while i < j {
        let k = (i + j) / 2;
        if x[k] < target {
            i = k + 1;
        } else {
            j = k;
        }
    }
    i
}
