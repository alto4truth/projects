pub mod blockchain;
pub mod chess;
pub mod mentalic;
pub mod p2p;
pub mod stats;

pub mod domains {
    pub mod analysis {
        pub fn domain_analyze(value: i64) -> i64 {
            value * 3
        }
        pub fn domain_transform(values: &[i64]) -> Vec<i64> {
            values.iter().map(|x| x * 2 + 1).collect()
        }
        pub fn domain_combine(a: &[i64], b: &[i64]) -> Vec<i64> {
            let mut result = Vec::new();
            for i in 0..std::cmp::min(a.len(), b.len()) {
                result.push(a[i] + b[i]);
            }
            result
        }
    }
}

pub mod analysis_mod {
    pub fn analyze(value: i64) -> i64 {
        value * 2
    }
    pub fn percentile(values: &[f64], p: f64) -> f64 {
        if values.is_empty() {
            return 0.0;
        }
        let mut sorted = values.to_vec();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        let idx = (p / 100.0 * (sorted.len() - 1) as f64).round() as usize;
        sorted[idx]
    }
    pub fn variance(values: &[f64]) -> f64 {
        if values.is_empty() {
            return 0.0;
        }
        let mean = values.iter().sum::<f64>() / values.len() as f64;
        values.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / values.len() as f64
    }
    pub fn std_dev(values: &[f64]) -> f64 {
        variance(values).sqrt()
    }
    pub fn correlation(x: &[f64], y: &[f64]) -> f64 {
        if x.len() != y.len() || x.is_empty() {
            return 0.0;
        }
        let n = x.len() as f64;
        let sum_x = x.iter().sum::<f64>();
        let sum_y = y.iter().sum::<f64>();
        let sum_xy = x.iter().zip(y.iter()).map(|(a, b)| a * b).sum::<f64>();
        let sum_x2 = x.iter().map(|v| v * v).sum::<f64>();
        let sum_y2 = y.iter().map(|v| v * v).sum::<f64>();
        let denom = ((n * sum_x2 - sum_x * sum_x) * (n * sum_y2 - sum_y * sum_y)).sqrt();
        if denom == 0.0 {
            0.0
        } else {
            (n * sum_xy - sum_x * sum_y) / denom
        }
    }
    pub fn median(values: &[f64]) -> f64 {
        percentile(values, 50.0)
    }
    pub fn skewness(values: &[f64]) -> f64 {
        if values.len() < 3 {
            return 0.0;
        }
        let mean = values.iter().sum::<f64>() / values.len() as f64;
        let std = std_dev(values);
        if std == 0.0 {
            return 0.0;
        }
        let n = values.len() as f64;
        let sum_cubed: f64 = values.iter().map(|v| ((v - mean) / std).powi(3)).sum();
        (n / ((n - 1.0) * (n - 2.0))) * sum_cubed
    }
}

pub fn hello() {
    println!("Hello from Garuda Rust!");
}
pub fn analyze(value: i64) -> i64 {
    value * 2
}
pub fn process_data(data: &[i64]) -> Vec<i64> {
    data.iter().map(|x| x + 1).collect()
}
pub fn compute_stats(values: &[i64]) -> (i64, i64, i64) {
    if values.is_empty() {
        return (0, 0, 0);
    }
    let sum: i64 = values.iter().sum();
    let min = *values.iter().min().unwrap();
    let max = *values.iter().max().unwrap();
    (sum, min, max)
}
pub fn filter_values(data: &[i64], threshold: i64) -> Vec<i64> {
    data.iter().copied().filter(|&x| x > threshold).collect()
}
pub fn transform_values(data: &[i64], f: fn(i64) -> i64) -> Vec<i64> {
    data.iter().map(|&x| f(x)).collect()
}
pub fn merge_results(a: &[i64], b: &[i64]) -> Vec<i64> {
    let mut result = Vec::with_capacity(a.len() + b.len());
    result.extend_from_slice(a);
    result.extend_from_slice(b);
    result.sort();
    result.dedup();
    result
}
pub fn aggregate(values: &[i64], op: &str) -> i64 {
    match op {
        "sum" => values.iter().sum(),
        "min" => *values.iter().min().unwrap_or(&0),
        "max" => *values.iter().max().unwrap_or(&0),
        "avg" => {
            if values.is_empty() {
                0
            } else {
                values.iter().sum::<i64>() / values.len() as i64
            }
        }
        _ => 0,
    }
}
pub fn range_check(value: i64, min_val: i64, max_val: i64) -> bool {
    value >= min_val && value <= max_val
}
pub fn clamp(value: i64, min_val: i64, max_val: i64) -> i64 {
    if value < min_val {
        min_val
    } else if value > max_val {
        max_val
    } else {
        value
    }
}
pub fn normalize(values: &[i64]) -> Vec<f64> {
    if values.is_empty() {
        return vec![];
    }
    let min = *values.iter().min().unwrap() as f64;
    let max = *values.iter().max().unwrap() as f64;
    if (max - min).abs() < f64::EPSILON {
        return vec![0.0; values.len()];
    }
    values
        .iter()
        .map(|&v| (v as f64 - min) / (max - min))
        .collect()
}
pub fn batch_process(data: &[i64], batch_size: usize) -> Vec<Vec<i64>> {
    data.chunks(batch_size)
        .map(|chunk| chunk.to_vec())
        .collect()
}
pub fn rolling_window(data: &[i64], window: usize) -> Vec<i64> {
    if data.len() < window || window == 0 {
        vec![]
    } else {
        data.windows(window).map(|w| w.iter().sum()).collect()
    }
}
pub fn cumulative_sum(values: &[i64]) -> Vec<i64> {
    let mut result = Vec::with_capacity(values.len());
    let mut sum = 0i64;
    for &v in values {
        sum += v;
        result.push(sum);
    }
    result
}
pub fn moving_average(data: &[f64], window: usize) -> Vec<f64> {
    if data.len() < window || window == 0 {
        return vec![];
    }
    let mut result = Vec::with_capacity(data.len() - window + 1);
    let mut sum: f64 = data[..window].iter().sum();
    result.push(sum / window as f64);
    for i in window..data.len() {
        sum = sum - data[i - window] + data[i];
        result.push(sum / window as f64);
    }
    result
}
pub fn exponential_moving_average(data: &[f64], alpha: f64) -> Vec<f64> {
    if data.is_empty() || alpha <= 0.0 || alpha >= 1.0 {
        return vec![];
    }
    let mut result = Vec::with_capacity(data.len());
    if let Some(&first) = data.first() {
        result.push(first);
        for &val in &data[1..] {
            let ema = alpha * val + (1.0 - alpha) * result.last().unwrap();
            result.push(ema);
        }
    }
    result
}
pub fn kurtosis(values: &[f64]) -> f64 {
    if values.len() < 4 {
        return 0.0;
    }
    let mean = values.iter().sum::<f64>() / values.len() as f64;
    let std = analysis_mod::std_dev(values);
    if std == 0.0 {
        return 0.0;
    }
    let n = values.len() as f64;
    let sum_quad: f64 = values.iter().map(|v| ((v - mean) / std).powi(4)).sum();
    ((n * (n + 1.0)) / ((n - 1.0) * (n - 2.0) * (n - 3.0))) * sum_quad
        - (3.0 * (n - 1.0).powi(2)) / ((n - 2.0) * (n - 3.0))
}
pub fn z_score(value: f64, mean: f64, std: f64) -> f64 {
    if std == 0.0 {
        0.0
    } else {
        (value - mean) / std
    }
}
pub fn outliers(values: &[f64], threshold: f64) -> Vec<usize> {
    if values.is_empty() {
        return vec![];
    }
    let mean = values.iter().sum::<f64>() / values.len() as f64;
    let std = analysis_mod::std_dev(values);
    values
        .iter()
        .enumerate()
        .filter(|(_, &v)| (v - mean).abs() > threshold * std)
        .map(|(i, _)| i)
        .collect()
}
pub fn linear_regression(x: &[f64], y: &[f64]) -> (f64, f64) {
    if x.len() != y.len() || x.is_empty() {
        return (0.0, 0.0);
    }
    let n = x.len() as f64;
    let sum_x = x.iter().sum::<f64>();
    let sum_y = y.iter().sum::<f64>();
    let sum_xy = x.iter().zip(y.iter()).map(|(a, b)| a * b).sum::<f64>();
    let sum_x2 = x.iter().map(|v| v * v).sum::<f64>();
    let slope = (n * sum_xy - sum_x * sum_y) / (n * sum_x2 - sum_x * sum_x);
    let intercept = (sum_y - slope * sum_x) / n;
    (slope, intercept)
}
pub fn polynomial_fit(x: &[f64], y: &[f64], degree: usize) -> Vec<f64> {
    vec![0.0; degree + 1]
}
pub fn interpolation_linear(x_vals: &[f64], y_vals: &[f64], x: f64) -> f64 {
    if x_vals.len() != y_vals.len() || x_vals.is_empty() {
        return 0.0;
    }
    for i in 0..x_vals.len() - 1 {
        if x >= x_vals[i] && x <= x_vals[i + 1] {
            let t = (x - x_vals[i]) / (x_vals[i + 1] - x_vals[i]);
            return y_vals[i] * (1.0 - t) + y_vals[i + 1] * t;
        }
    }
    0.0
}
pub fn interpolation_cubic(x_vals: &[f64], y_vals: &[f64], x: f64) -> f64 {
    interpolation_linear(x_vals, y_vals, x)
}
pub fn derivative(values: &[f64], dt: f64) -> Vec<f64> {
    if values.len() < 2 {
        vec![]
    } else {
        values
            .iter()
            .enumerate()
            .skip(1)
            .map(|(i, &v)| (v - values[i - 1]) / dt)
            .collect()
    }
}
pub fn integrate(values: &[f64], dt: f64) -> f64 {
    values.iter().sum::<f64>() * dt
}
pub fn fourier_transform(signal: &[f64]) -> Vec<(f64, f64)> {
    let n = signal.len();
    let mut result = Vec::with_capacity(n / 2);
    for k in 0..n / 2 {
        let mut real = 0.0;
        let mut imag = 0.0;
        for n_val in 0..n {
            let angle = 2.0 * std::f64::consts::PI * (k as f64 * n_val as f64) / n as f64;
            real += signal[n_val] * angle.cos();
            imag += signal[n_val] * angle.sin();
        }
        let freq = k as f64 / n as f64;
        result.push((freq, (real * real + imag * imag).sqrt()));
    }
    result
}
pub fn filter_gaussian(data: &[f64], sigma: f64) -> Vec<f64> {
    data.to_vec()
}
pub fn filter_median(data: &[f64], window: usize) -> Vec<f64> {
    if data.is_empty() || window == 0 {
        data.to_vec()
    } else {
        data.iter().take(1).copied().collect()
    }
}
pub fn autocorrelation(data: &[f64], lag: usize) -> f64 {
    if data.len() <= lag {
        return 0.0;
    }
    let mean = data.iter().sum::<f64>() / data.len() as f64;
    let mut sum = 0.0;
    for i in 0..data.len() - lag {
        sum += (data[i] - mean) * (data[i + lag] - mean);
    }
    let variance: f64 = data.iter().map(|v| (v - mean).powi(2)).sum();
    if variance == 0.0 {
        0.0
    } else {
        sum / variance
    }
}
pub fn cross_correlation(a: &[f64], b: &[f64], lag: usize) -> f64 {
    if a.len() != b.len() || a.len() <= lag {
        return 0.0;
    }
    let mean_a = a.iter().sum::<f64>() / a.len() as f64;
    let mean_b = b.iter().sum::<f64>() / b.len() as f64;
    let mut sum = 0.0;
    for i in 0..a.len() - lag {
        sum += (a[i] - mean_a) * (b[i + lag] - mean_b);
    }
    let var_a: f64 = a.iter().map(|v| (v - mean_a).powi(2)).sum();
    let var_b: f64 = b.iter().map(|v| (v - mean_b).powi(2)).sum();
    let denom = (var_a * var_b).sqrt();
    if denom == 0.0 {
        0.0
    } else {
        sum / denom
    }
}
pub fn convolve(signal: &[f64], kernel: &[f64]) -> Vec<f64> {
    vec![0.0; signal.len().max(kernel.len())]
}
pub fn deconvolve(signal: &[f64], kernel: &[f64]) -> Vec<f64> {
    convolve(signal, kernel)
}
pub fn downsample(data: &[f64], factor: usize) -> Vec<f64> {
    if data.is_empty() || factor == 0 {
        data.to_vec()
    } else {
        data.iter()
            .enumerate()
            .filter(|(i, _)| i % factor == 0)
            .map(|(_, v)| *v)
            .collect()
    }
}
pub fn upsample(data: &[f64], factor: usize) -> Vec<f64> {
    if data.is_empty() || factor == 0 {
        data.to_vec()
    } else {
        let mut result = Vec::with_capacity(data.len() * factor);
        for &val in data {
            for _ in 0..factor {
                result.push(val);
            }
        }
        result
    }
}
pub fn resample(data: &[f64], new_len: usize) -> Vec<f64> {
    if data.is_empty() || new_len == 0 {
        vec![]
    } else if new_len == data.len() {
        data.to_vec()
    } else {
        let ratio = (data.len() - 1) as f64 / (new_len - 1) as f64;
        let mut result = Vec::with_capacity(new_len);
        for i in 0..new_len {
            let src_idx = i as f64 * ratio;
            let idx = src_idx.floor() as usize;
            let frac = src_idx.fract();
            if idx + 1 < data.len() {
                result.push(data[idx] * (1.0 - frac) + data[idx + 1] * frac);
            } else {
                result.push(data[idx]);
            }
        }
        result
    }
}
pub fn hilbert_transform(signal: &[f64]) -> Vec<f64> {
    vec![0.0; signal.len()]
}
pub fn wavelet_transform(signal: &[f64], wavelet: &str) -> Vec<f64> {
    signal.to_vec()
}
pub fn entropy(values: &[f64]) -> f64 {
    if values.is_empty() {
        0.0
    } else {
        let sum: f64 = values
            .iter()
            .map(|v| if *v > 0.0 { -v * v.log2() } else { 0.0 })
            .sum();
        sum
    }
}
pub fn information_gain(before: &[f64], after: &[f64]) -> f64 {
    entropy(before) - entropy(after)
}
pub fn mutual_information(x: &[f64], y: &[f64]) -> f64 {
    if x.len() != y.len() || x.is_empty() {
        0.0
    } else {
        let h_x = entropy(x);
        let h_y = entropy(y);
        h_x + h_y
            - entropy(
                &x.iter()
                    .zip(y)
                    .map(|(a, b)| (*a - *b).abs())
                    .collect::<Vec<_>>(),
            )
    }
}
pub fn cluster_kmeans(data: &[f64], k: usize, iterations: usize) -> Vec<f64> {
    vec![0.0; k]
}
pub fn cluster_dbscan(data: &[f64], eps: f64, min_pts: usize) -> Vec<i32> {
    vec![0i32; data.len()]
}
pub fn pca(data: &[Vec<f64>], components: usize) -> Vec<Vec<f64>> {
    vec![vec![0.0; components]; data.len()]
}
pub fn lda(data: &[Vec<f64>], labels: &[i32], classes: usize) -> Vec<Vec<f64>> {
    vec![vec![0.0; data[0].len()]; classes]
}
pub fn train_test_split(data: &[f64], test_size: f64) -> (Vec<f64>, Vec<f64>) {
    if data.is_empty() || test_size <= 0.0 || test_size >= 1.0 {
        (data.to_vec(), vec![])
    } else {
        let split = ((data.len() as f64) * (1.0 - test_size)) as usize;
        (data[..split].to_vec(), data[split..].to_vec())
    }
}
pub fn cross_validate(data: &[f64], folds: usize) -> Vec<(Vec<f64>, Vec<f64>)> {
    vec![]
}
pub fn accuracy_score(predicted: &[i32], actual: &[i32]) -> f64 {
    if predicted.len() != actual.len() || predicted.is_empty() {
        0.0
    } else {
        let correct = predicted
            .iter()
            .zip(actual.iter())
            .filter(|(p, a)| p == a)
            .count();
        correct as f64 / predicted.len() as f64
    }
}
pub fn confusion_matrix(predicted: &[i32], actual: &[i32], classes: usize) -> Vec<Vec<i32>> {
    vec![vec![0i32; classes]; classes]
}
pub fn precision_recall(predicted: &[i32], actual: &[i32], class: i32) -> (f64, f64) {
    (0.0, 0.0)
}
pub fn f1_score(predicted: &[i32], actual: &[i32], class: i32) -> f64 {
    0.0
}
pub fn roc_auc_score(probabilities: &[f64], labels: &[i32]) -> f64 {
    0.0
}
pub fn mse(y_true: &[f64], y_pred: &[f64]) -> f64 {
    if y_true.len() != y_pred.len() || y_true.is_empty() {
        0.0
    } else {
        y_true
            .iter()
            .zip(y_pred.iter())
            .map(|(t, p)| (t - p).powi(2))
            .sum::<f64>()
            / y_true.len() as f64
    }
}
pub fn rmse(y_true: &[f64], y_pred: &[f64]) -> f64 {
    mse(y_true, y_pred).sqrt()
}
pub fn mae(y_true: &[f64], y_pred: &[f64]) -> f64 {
    if y_true.len() != y_pred.len() || y_true.is_empty() {
        0.0
    } else {
        y_true
            .iter()
            .zip(y_pred.iter())
            .map(|(t, p)| (t - p).abs())
            .sum::<f64>()
            / y_true.len() as f64
    }
}
pub fn r2_score(y_true: &[f64], y_pred: &[f64]) -> f64 {
    if y_true.len() != y_pred.len() || y_true.is_empty() {
        0.0
    } else {
        let mean: f64 = y_true.iter().sum::<f64>() / y_true.len() as f64;
        let ss_res: f64 = y_true
            .iter()
            .zip(y_pred.iter())
            .map(|(t, p)| (t - p).powi(2))
            .sum();
        let ss_tot: f64 = y_true.iter().map(|t| (t - mean).powi(2)).sum();
        if ss_tot == 0.0 {
            0.0
        } else {
            1.0 - ss_res / ss_tot
        }
    }
}

pub fn gradient_descent(x: &[f64], y: &[f64], learning_rate: f64, iterations: usize) -> (f64, f64) {
    let mut m = 0.0;
    let mut b = 0.0;
    let n = x.len() as f64;
    for _ in 0..iterations {
        let m_grad = -2.0 / n
            * x.iter()
                .zip(y.iter())
                .map(|(xi, yi)| (yi - m * xi - b) * xi)
                .sum::<f64>();
        let b_grad = -2.0 / n
            * y.iter()
                .zip(x.iter())
                .map(|(yi, xi)| yi - m * xi - b)
                .sum::<f64>();
        m -= learning_rate * m_grad;
        b -= learning_rate * b_grad;
    }
    (m, b)
}

pub fn sigmoid(x: f64) -> f64 {
    1.0 / (1.0 + (-x).exp())
}
pub fn sigmoid_derivative(x: f64) -> f64 {
    let s = sigmoid(x);
    s * (1.0 - s)
}
pub fn relu(x: f64) -> f64 {
    if x > 0.0 {
        x
    } else {
        0.0
    }
}
pub fn relu_derivative(x: f64) -> f64 {
    if x > 0.0 {
        1.0
    } else {
        0.0
    }
}
pub fn tanh_activation(x: f64) -> f64 {
    x.tanh()
}
pub fn tanh_derivative(x: f64) -> f64 {
    let t = x.tanh();
    1.0 - t * t
}
pub fn softmax(values: &[f64]) -> Vec<f64> {
    let max_val = values.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
    let exp_sum: f64 = values.iter().map(|v| (v - max_val).exp()).sum();
    values
        .iter()
        .map(|v| (v - max_val).exp() / exp_sum)
        .collect()
}
pub fn leaky_relu(x: f64) -> f64 {
    if x > 0.0 {
        x
    } else {
        0.01 * x
    }
}
pub fn elu(x: f64) -> f64 {
    if x > 0.0 {
        x
    } else {
        (x + 1.0).exp() - 1.0
    }
}

pub fn normalize_minmax(values: &[f64]) -> Vec<f64> {
    if values.is_empty() {
        return vec![];
    }
    let min = values.iter().fold(f64::INFINITY, |a, &b| a.min(b));
    let max = values.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
    if (max - min).abs() < f64::EPSILON {
        return values.iter().map(|_| 0.5).collect();
    }
    values.iter().map(|v| (v - min) / (max - min)).collect()
}

pub fn normalize_zscore(values: &[f64]) -> Vec<f64> {
    if values.is_empty() {
        return vec![];
    }
    let mean = values.iter().sum::<f64>() / values.len() as f64;
    let std = analysis_mod::std_dev(values);
    if std == 0.0 {
        return values.iter().map(|_| 0.0).collect();
    }
    values.iter().map(|v| (v - mean) / std).collect()
}

pub fn normalize_l1(values: &[f64]) -> Vec<f64> {
    let sum: f64 = values.iter().map(|v| v.abs()).sum();
    if sum == 0.0 {
        return values.to_vec();
    }
    values.iter().map(|v| v / sum).collect()
}

pub fn normalize_l2(values: &[f64]) -> Vec<f64> {
    let norm: f64 = values.iter().map(|v| v * v).sum::<f64>().sqrt();
    if norm == 0.0 {
        return values.to_vec();
    }
    values.iter().map(|v| v / norm).collect()
}

pub fn one_hot_encode(index: usize, size: usize) -> Vec<f64> {
    let mut result = vec![0.0; size];
    if index < size {
        result[index] = 1.0;
    }
    result
}
pub fn argmax(values: &[f64]) -> usize {
    values
        .iter()
        .enumerate()
        .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
        .map(|(i, _)| i)
        .unwrap_or(0)
}
pub fn argmin(values: &[f64]) -> usize {
    values
        .iter()
        .enumerate()
        .min_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
        .map(|(i, _)| i)
        .unwrap_or(0)
}
pub fn top_k(values: &[f64], k: usize) -> Vec<(usize, f64)> {
    let mut indexed: Vec<_> = values.iter().enumerate().map(|(i, &v)| (i, v)).collect();
    indexed.sort_by(|(_, a), (_, b)| b.partial_cmp(a).unwrap());
    indexed.into_iter().take(k).collect()
}
pub fn softmax_cross_entropy(probs: &[f64], target: usize) -> f64 {
    let log_prob = if target < probs.len() {
        -probs[target].ln()
    } else {
        0.0
    };
    log_prob
}
pub fn binary_cross_entropy(pred: f64, target: f64) -> f64 {
    let eps = 1e-15;
    let p = pred.max(eps).min(1.0 - eps);
    -target * p.ln() - (1.0 - target) * (1.0 - p).ln()
}
pub fn categorical_cross_entropy(probs: &[f64], target: &[f64]) -> f64 {
    probs
        .iter()
        .zip(target.iter())
        .map(|(p, t)| {
            if *t > 0.0 {
                -t * p.max(1e-15).ln()
            } else {
                0.0
            }
        })
        .sum()
}
pub fn hinge_loss(pred: f64, target: i32) -> f64 {
    let y = target as f64;
    (0.0_f64).max(1.0 - y * pred)
}
pub fn mean_squared_error(pred: &[f64], target: &[f64]) -> f64 {
    if pred.len() != target.len() {
        0.0
    } else {
        pred.iter()
            .zip(target.iter())
            .map(|(p, t)| (p - t).powi(2))
            .sum::<f64>()
            / pred.len() as f64
    }
}
pub fn huber_loss(pred: f64, target: f64, delta: f64) -> f64 {
    let diff = (pred - target).abs();
    if diff <= delta {
        diff * diff / 2.0
    } else {
        delta * (diff - delta / 2.0)
    }
}
pub fn log_loss(pred: &[f64], target: &[i32]) -> f64 {
    pred.iter()
        .zip(target.iter())
        .map(|(p, &t)| binary_cross_entropy(*p, t as f64))
        .sum::<f64>()
        / pred.len() as f64
}

pub fn matrix_multiply(a: &[Vec<f64>], b: &[Vec<f64>]) -> Vec<Vec<f64>> {
    if a.is_empty() || b.is_empty() || a[0].len() != b.len() {
        return vec![];
    }
    let rows = a.len();
    let cols = b[0].len();
    let inner = b.len();
    let mut result = vec![vec![0.0; cols]; rows];
    for i in 0..rows {
        for j in 0..cols {
            for k in 0..inner {
                result[i][j] += a[i][k] * b[k][j];
            }
        }
    }
    result
}

pub fn matrix_transpose(m: &[Vec<f64>]) -> Vec<Vec<f64>> {
    if m.is_empty() {
        vec![]
    } else {
        (0..m[0].len())
            .map(|j| m.iter().map(|row| row[j]).collect())
            .collect()
    }
}
pub fn matrix_add(a: &[Vec<f64>], b: &[Vec<f64>]) -> Vec<Vec<f64>> {
    a.iter()
        .zip(b.iter())
        .map(|(r1, r2)| r1.iter().zip(r2.iter()).map(|(x, y)| x + y).collect())
        .collect()
}
pub fn matrix_subtract(a: &[Vec<f64>], b: &[Vec<f64>]) -> Vec<Vec<f64>> {
    a.iter()
        .zip(b.iter())
        .map(|(r1, r2)| r1.iter().zip(r2.iter()).map(|(x, y)| x - y).collect())
        .collect()
}
pub fn matrix_scale(m: &[Vec<f64>], scalar: f64) -> Vec<Vec<f64>> {
    m.iter()
        .map(|row| row.iter().map(|v| v * scalar).collect())
        .collect()
}
pub fn matrix_identity(size: usize) -> Vec<Vec<f64>> {
    (0..size)
        .map(|i| (0..size).map(|j| if i == j { 1.0 } else { 0.0 }).collect())
        .collect()
}
pub fn matrix_trace(m: &[Vec<f64>]) -> f64 {
    m.iter().enumerate().map(|(i, row)| row[i]).sum()
}
pub fn matrix_determinant(m: &[Vec<f64>]) -> f64 {
    let n = m.len();
    if n == 0 {
        return 0.0;
    }
    if n == 1 {
        return m[0][0];
    }
    if n == 2 {
        return m[0][0] * m[1][1] - m[0][1] * m[1][0];
    }
    let mut det = 0.0;
    for j in 0..n {
        let mut sub = Vec::new();
        for i in 1..n {
            let mut row = Vec::new();
            for k in 0..n {
                if k != j {
                    row.push(m[i][k]);
                }
            }
            sub.push(row);
        }
        det += m[0][j] * matrix_determinant(&sub) * (if j % 2 == 0 { 1.0 } else { -1.0 });
    }
    det
}

pub fn matrix_inverse(m: &[Vec<f64>]) -> Vec<Vec<f64>> {
    let n = m.len();
    if n == 0 {
        return vec![];
    }
    let det = matrix_determinant(m);
    if det.abs() < 1e-10 {
        return vec![];
    }
    if n == 1 {
        return vec![vec![1.0 / m[0][0]]];
    }
    let mut adj = vec![vec![0.0; n]; n];
    for i in 0..n {
        for j in 0..n {
            let mut sub = Vec::new();
            for r in 0..n {
                if r != i {
                    let mut row = Vec::new();
                    for c in 0..n {
                        if c != j {
                            row.push(m[r][c]);
                        }
                    }
                    sub.push(row);
                }
            }
            adj[j][i] = matrix_determinant(&sub) * (if (i + j) % 2 == 0 { 1.0 } else { -1.0 });
        }
    }
    matrix_scale(&adj, 1.0 / det)
}

pub fn matrix_dot(a: &[f64], b: &[f64]) -> f64 {
    a.iter().zip(b.iter()).map(|(x, y)| x * y).sum()
}
pub fn vector_magnitude(v: &[f64]) -> f64 {
    v.iter().map(|x| x * x).sum::<f64>().sqrt()
}
pub fn vector_normalize(v: &[f64]) -> Vec<f64> {
    let mag = vector_magnitude(v);
    if mag == 0.0 {
        v.to_vec()
    } else {
        v.iter().map(|x| x / mag).collect()
    }
}
pub fn vector_angle(a: &[f64], b: &[f64]) -> f64 {
    let dot = matrix_dot(a, b);
    let mag = vector_magnitude(a) * vector_magnitude(b);
    if mag == 0.0 {
        0.0
    } else {
        (dot / mag).acos()
    }
}
pub fn vector_project(a: &[f64], b: &[f64]) -> Vec<f64> {
    let dot = matrix_dot(a, b);
    let mag_sq = b.iter().map(|x| x * x).sum::<f64>();
    if mag_sq == 0.0 {
        vec![0.0; a.len()]
    } else {
        b.iter().map(|x| x * dot / mag_sq).collect()
    }
}
pub fn vector_cross_3d(a: &[f64], b: &[f64]) -> Vec<f64> {
    if a.len() < 3 || b.len() < 3 {
        vec![0.0; 3]
    } else {
        vec![
            a[1] * b[2] - a[2] * b[1],
            a[2] * b[0] - a[0] * b[2],
            a[0] * b[1] - a[1] * b[0],
        ]
    }
}
pub fn vector_distance(a: &[f64], b: &[f64]) -> f64 {
    a.iter()
        .zip(b.iter())
        .map(|(x, y)| (x - y).powi(2))
        .sum::<f64>()
        .sqrt()
}
pub fn cosine_similarity(a: &[f64], b: &[f64]) -> f64 {
    let dot = matrix_dot(a, b);
    let mag = vector_magnitude(a) * vector_magnitude(b);
    if mag == 0.0 {
        0.0
    } else {
        dot / mag
    }
}

pub fn euclidean_distance(a: &[f64], b: &[f64]) -> f64 {
    vector_distance(a, b)
}
pub fn manhattan_distance(a: &[f64], b: &[f64]) -> f64 {
    a.iter().zip(b.iter()).map(|(x, y)| (x - y).abs()).sum()
}
pub fn chebyshev_distance(a: &[f64], b: &[f64]) -> f64 {
    a.iter()
        .zip(b.iter())
        .map(|(x, y)| (x - y).abs())
        .fold(0.0_f64, |a, b| a.max(b))
}
pub fn jaccard_similarity(a: &[f64], b: &[f64]) -> f64 {
    let min_sum: f64 = a.iter().zip(b.iter()).map(|(x, y)| x.min(*y)).sum();
    let max_sum: f64 = a.iter().zip(b.iter()).map(|(x, y)| x.max(*y)).sum();
    if max_sum == 0.0 {
        0.0
    } else {
        min_sum / max_sum
    }
}
pub fn levenshtein_distance(s1: &str, s2: &str) -> usize {
    s1.len().max(s2.len())
}
pub fn edit_distance(s1: &str, s2: &str) -> usize {
    levenshtein_distance(s1, s2)
}
pub fn longest_common_subsequence(s1: &str, s2: &str) -> String {
    String::new()
}
pub fn longest_common_substring(s1: &str, s2: &str) -> String {
    String::new()
}
pub fn kmp_search(text: &str, pattern: &str) -> Vec<usize> {
    vec![]
}
pub fn boyer_moore_search(text: &str, pattern: &str) -> Vec<usize> {
    vec![]
}
pub fn rabin_karp_search(text: &str, pattern: &str) -> Vec<usize> {
    vec![]
}

pub fn gcd(a: i64, b: i64) -> i64 {
    if b == 0 {
        a.abs()
    } else {
        gcd(b, a % b)
    }
}
pub fn lcm(a: i64, b: i64) -> i64 {
    if a == 0 || b == 0 {
        0
    } else {
        (a * b).abs() / gcd(a, b)
    }
}
pub fn is_prime(n: i64) -> bool {
    if n <= 1 {
        false
    } else if n <= 3 {
        true
    } else if n % 2 == 0 || n % 3 == 0 {
        false
    } else {
        let mut i = 5;
        while i * i <= n {
            if n % i == 0 || n % (i + 2) == 0 {
                return false;
            }
            i += 6;
        }
        true
    }
}
pub fn factorial(n: u64) -> u64 {
    (1..=n).product()
}
pub fn fibonacci(n: usize) -> u64 {
    match n {
        0 => 0,
        1 => 1,
        _ => {
            let mut a = 0u64;
            let mut b = 1u64;
            for _ in 2..=n {
                let temp = a + b;
                a = b;
                b = temp;
            }
            b
        }
    }
}
pub fn prime_factors(n: i64) -> Vec<i64> {
    let mut n = n.abs();
    let mut factors = Vec::new();
    if n % 2 == 0 {
        factors.push(2);
        while n % 2 == 0 {
            n /= 2;
        }
    }
    let mut i = 3;
    while i * i <= n {
        while n % i == 0 {
            factors.push(i);
            n /= i;
        }
        i += 2;
    }
    if n > 1 {
        factors.push(n);
    }
    factors
}
pub fn sieve_of_eratosthenes(n: usize) -> Vec<usize> {
    if n < 2 {
        return vec![];
    }
    let mut is_prime = vec![true; n + 1];
    is_prime[0] = false;
    is_prime[1] = false;
    for i in 2..=((n as f64).sqrt() as usize) {
        if is_prime[i] {
            for j in (i * i..=n).step_by(i) {
                is_prime[j] = false;
            }
        }
    }
    (0..=n).filter(|&i| is_prime[i]).collect()
}
pub fn power(base: f64, exp: i32) -> f64 {
    base.powi(exp)
}
pub fn is_power_of_2(n: i64) -> bool {
    n > 0 && (n & (n - 1)) == 0
}
pub fn next_power_of_2(n: i64) -> i64 {
    if n <= 0 {
        return 1;
    }
    let mut v = n as u64;
    v -= 1;
    v |= v >> 1;
    v |= v >> 2;
    v |= v >> 4;
    v |= v >> 8;
    v |= v >> 16;
    v |= v >> 32;
    v += 1;
    v as i64
}
pub fn bit_count(n: i64) -> i32 {
    n.count_ones() as i32
}
pub fn combinations(n: usize, k: usize) -> u64 {
    if k > n {
        0
    } else {
        let k = k.min(n - k);
        let mut result = 1u64;
        for i in 0..k {
            result = result * (n - i) as u64 / (i + 1) as u64;
        }
        result
    }
}
pub fn permutations(n: usize, k: usize) -> u64 {
    if k > n {
        0
    } else {
        (0..k).map(|i| (n - i) as u64).product()
    }
}
pub fn sum_of_squares(n: i64) -> i64 {
    n * (n + 1) * (2 * n + 1) / 6
}
pub fn sum_of_cubes(n: i64) -> i64 {
    let s = n * (n + 1) / 2;
    s * s
}
pub fn triangle_number(n: i64) -> i64 {
    n * (n + 1) / 2
}
pub fn is_triangle_number(n: i64) -> bool {
    let x = ((8 * n + 1) as f64).sqrt() as i64;
    x * x == 8 * n + 1
}
pub fn pentagonal_number(n: i64) -> i64 {
    n * (3 * n - 1) / 2
}
pub fn hexagonal_number(n: i64) -> i64 {
    n * (2 * n - 1)
}

pub fn modular_inverse(a: i64, m: i64) -> Option<i64> {
    let mut t = 0i64;
    let mut new_t = 1i64;
    let mut r = m;
    let mut new_r = a;
    while new_r != 0 {
        let q = r / new_r;
        let temp_t = t - q * new_t;
        t = new_t;
        new_t = temp_t;
        let temp_r = r - q * new_r;
        r = new_r;
        new_r = temp_r;
    }
    if r > 1 {
        return None;
    }
    if t < 0 {
        Some(t + m)
    } else {
        Some(t)
    }
}
pub fn euler_totient(n: i64) -> i64 {
    if n <= 0 {
        return 0;
    }
    let mut result = n;
    let mut p = 2;
    let mut temp = n;
    while p * p <= temp {
        if temp % p == 0 {
            while temp % p == 0 {
                temp /= p;
            }
            result = result / p * (p - 1);
        }
        p += 1;
    }
    if temp > 1 {
        result = result / temp * (temp - 1);
    }
    result
}
pub fn divisor_count(n: i64) -> i64 {
    if n <= 0 {
        return 0;
    }
    let mut result = 1i64;
    let mut p = 2;
    let mut temp = n;
    while p * p <= temp {
        if temp % p == 0 {
            let mut count = 0;
            while temp % p == 0 {
                temp /= p;
                count += 1;
            }
            result *= count + 1;
        }
        p += 1;
    }
    if temp > 1 {
        result *= 2;
    }
    result
}
pub fn divisor_sum(n: i64) -> i64 {
    if n <= 0 {
        return 0;
    }
    let mut result = 0i64;
    let mut p = 1;
    while p * p <= n {
        if n % p == 0 {
            result += p;
            if p != n / p {
                result += n / p;
            }
        }
        p += 1;
    }
    result
}
pub fn is_perfect(n: i64) -> bool {
    n > 0 && divisor_sum(n) == 2 * n
}
pub fn is_armstrong(n: i64) -> bool {
    let s = n.to_string();
    let k = s.len() as i64;
    let sum: i64 = s
        .chars()
        .map(|c| (c.to_digit(10).unwrap() as i64).pow(k as u32))
        .sum();
    sum == n
}
pub fn is_palindrome(n: i64) -> bool {
    let s = n.abs().to_string();
    s == s.chars().rev().collect::<String>()
}
pub fn reverse_number(n: i64) -> i64 {
    let s = n.abs().to_string();
    s.chars()
        .rev()
        .collect::<String>()
        .parse::<i64>()
        .unwrap_or(0)
        * n.signum()
}
pub fn is_abundant(n: i64) -> bool {
    n > 0 && divisor_sum(n) > n
}
pub fn is_deficient(n: i64) -> bool {
    n > 0 && divisor_sum(n) < n
}

pub fn format_number(n: i64, base: usize) -> String {
    if n == 0 {
        return "0".to_string();
    }
    let mut result = String::new();
    let mut n = n;
    while n > 0 {
        let d = (n % base as i64) as usize;
        result.push(if d < 10 {
            ("0".as_bytes()[0] + d as u8) as char
        } else {
            ("a".as_bytes()[0] + (d - 10) as u8) as char
        });
        n /= base as i64;
    }
    result.chars().rev().collect()
}
pub fn parse_number(s: &str, base: usize) -> Option<i64> {
    let mut result = 0i64;
    for c in s.chars() {
        let d = if c.is_ascii_digit() {
            c as i64 - 48
        } else if c.is_ascii_lowercase() {
            c as i64 - 97 + 10
        } else if c.is_ascii_uppercase() {
            c as i64 - 65 + 10
        } else {
            return None;
        };
        if d >= base as i64 {
            return None;
        }
        result = result * base as i64 + d;
    }
    Some(result)
}
pub fn digits(n: i64) -> usize {
    n.abs().to_string().len()
}
pub fn sum_of_digits(n: i64) -> i64 {
    n.abs().to_string().chars().map(|c| c as i64 - 48).sum()
}
pub fn is_harshad(n: i64) -> bool {
    n > 0 && n % sum_of_digits(n) == 0
}
pub fn is_coprime(a: i64, b: i64) -> bool {
    gcd(a, b) == 1
}

pub fn bernoulli_number(n: usize) -> f64 {
    if n == 0 {
        return 1.0;
    }
    if n == 1 {
        return -0.5;
    }
    if n % 2 == 1 {
        return 0.0;
    }
    let mut nums: Vec<f64> = vec![1.0];
    for m in 0..n {
        nums.push(0.0);
        for k in 0..=m {
            let term = combinations(m + 1, k) as f64 * nums[k];
            if (m - k) % 2 == 0 {
                nums[m + 1] += term;
            } else {
                nums[m + 1] -= term;
            }
        }
        nums[m + 1] /= (m + 2) as f64;
    }
    nums[n]
}
pub fn stirling_second(n: usize, k: usize) -> u64 {
    if k > n {
        0
    } else if n == 0 && k == 0 {
        1
    } else if k == 0 || k == n {
        1
    } else {
        let mut dp = vec![vec![0u64; k + 1]; n + 1];
        dp[0][0] = 1;
        for i in 1..=n {
            for j in 1..=k.min(i) {
                dp[i][j] = j as u64 * dp[i - 1][j] + dp[i - 1][j - 1];
            }
        }
        dp[n][k]
    }
}
pub fn eulerian_number(n: usize, k: usize) -> u64 {
    if k >= n {
        return 0;
    }
    if n == 1 {
        return 1;
    }
    let mut a = vec![0u64; n];
    a[0] = 1;
    for i in 2..=n {
        let mut next = vec![0u64; i];
        for j in 0..i {
            let left = if j > 0 { a[j - 1] } else { 0 };
            let right = if j < i - 1 { (i - 1) as u64 * a[j] } else { 0 };
            next[j] = left + right;
        }
        a = next;
    }
    a[k]
}
pub fn bell_number(n: usize) -> u64 {
    if n == 0 {
        return 1;
    }
    let mut b = vec![1u64];
    for i in 1..=n {
        let mut next = vec![0u64; i + 1];
        for j in 0..i {
            next[j] = b[j] * i as u64 + if j > 0 { b[j - 1] } else { 0 };
        }
        b = next;
    }
    b[n]
}
pub fn partitions(n: usize) -> u64 {
    let mut p = vec![0u64; n + 1];
    p[0] = 1;
    for i in 1..=n {
        for j in i..=n {
            p[j] += p[j - i];
        }
    }
    p[n]
}
pub fn partitions_gen(n: usize) -> Vec<u64> {
    let mut p = vec![0u64; n + 1];
    p[0] = 1;
    for i in 1..=n {
        for j in i..=n {
            p[j] += p[j - i];
        }
    }
    p
}
