#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RatioEdge {
    pub size: Option<usize>,
    pub ratio: usize,
    pub minimum_size: usize,
}

impl RatioEdge {
    pub const fn new(size: Option<usize>, ratio: usize, minimum_size: usize) -> Self {
        Self {
            size,
            ratio: if ratio == 0 { 1 } else { ratio },
            minimum_size: if minimum_size == 0 { 1 } else { minimum_size },
        }
    }
}

/// Divide total space to satisfy size, ratio, and minimum_size constraints.
pub fn ratio_resolve(total: usize, edges: &[RatioEdge]) -> Vec<usize> {
    let mut sizes: Vec<Option<usize>> = edges.iter().map(|e| e.size).collect();

    while sizes.iter().any(|s| s.is_none()) {
        let flexible: Vec<(usize, &RatioEdge)> = sizes
            .iter()
            .enumerate()
            .filter(|(_, s)| s.is_none())
            .map(|(i, _)| (i, &edges[i]))
            .collect();

        let fixed_sum: usize = sizes.iter().flatten().sum();
        if fixed_sum >= total {
            // No room left for flexible edges, assign minimums
            return sizes
                .iter()
                .enumerate()
                .map(|(i, s)| s.unwrap_or(edges[i].minimum_size))
                .collect();
        }

        let remaining = total - fixed_sum;
        let total_ratio: usize = flexible.iter().map(|(_, e)| e.ratio).sum();
        if total_ratio == 0 {
            return sizes
                .iter()
                .enumerate()
                .map(|(i, s)| s.unwrap_or(edges[i].minimum_size))
                .collect();
        }

        // Check if any edge will be less than its minimum
        let mut min_violation = false;
        for &(idx, edge) in &flexible {
            let portion = (remaining as f64 * edge.ratio as f64) / total_ratio as f64;
            if portion <= edge.minimum_size as f64 {
                sizes[idx] = Some(edge.minimum_size);
                min_violation = true;
                break;
            }
        }

        if !min_violation {
            // Distribute flexible space and compensate for rounding
            let mut remainder = 0.0;
            for &(idx, edge) in &flexible {
                let exact = (remaining as f64 * edge.ratio as f64) / total_ratio as f64 + remainder;
                let size = exact.floor() as usize;
                remainder = exact - size as f64;
                sizes[idx] = Some(size);
            }
            // Compensate any remaining whole units due to rounding
            let new_sum: usize = sizes.iter().flatten().sum();
            if new_sum < total {
                let diff = total - new_sum;
                if let Some(first_flex) = flexible.first()
                    && let Some(s) = sizes[first_flex.0].as_mut()
                {
                    *s += diff;
                }
            }
            break;
        }
    }

    sizes.into_iter().map(|s| s.unwrap_or(1)).collect()
}

/// Divide an integer total into parts based on ratios.
pub fn ratio_reduce(
    total: usize,
    ratios: &[usize],
    maximums: &[usize],
    values: &[usize],
) -> Vec<usize> {
    let mut total_ratio: usize = ratios
        .iter()
        .zip(maximums)
        .map(|(&r, &m)| if m > 0 { r } else { 0 })
        .sum();

    if total_ratio == 0 {
        return values.to_vec();
    }

    let mut total_remaining = total;
    let mut result = Vec::with_capacity(values.len());

    for (&ratio, (&maximum, &value)) in ratios.iter().zip(maximums.iter().zip(values.iter())) {
        if ratio > 0 && total_ratio > 0 {
            let distributed = maximum
                .min(((ratio * total_remaining) as f64 / total_ratio as f64).round() as usize);
            result.push(value.saturating_sub(distributed));
            total_remaining = total_remaining.saturating_sub(distributed);
            total_ratio = total_ratio.saturating_sub(ratio);
        } else {
            result.push(value);
        }
    }

    result
}

/// Distribute an integer total into parts based on ratios.
pub fn ratio_distribute(total: usize, ratios: &[usize], minimums: Option<&[usize]>) -> Vec<usize> {
    let mut total_ratio: usize = ratios.iter().sum();
    if total_ratio == 0 {
        return vec![0; ratios.len()];
    }

    let mut total_remaining = total;
    let mut result = Vec::with_capacity(ratios.len());
    let default_mins = vec![0; ratios.len()];
    let mins = minimums.unwrap_or(&default_mins);

    for (&ratio, &minimum) in ratios.iter().zip(mins.iter()) {
        let distributed = if total_ratio > 0 {
            let portion = ((ratio * total_remaining) as f64 / total_ratio as f64).ceil() as usize;
            minimum.max(portion).min(total_remaining)
        } else {
            total_remaining
        };
        result.push(distributed);
        total_ratio = total_ratio.saturating_sub(ratio);
        total_remaining = total_remaining.saturating_sub(distributed);
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ratio_resolve_simple() {
        let edges = vec![
            RatioEdge::new(None, 1, 1),
            RatioEdge::new(None, 1, 1),
            RatioEdge::new(None, 1, 1),
        ];
        let resolved = ratio_resolve(90, &edges);
        assert_eq!(resolved.len(), 3);
        assert_eq!(resolved.iter().sum::<usize>(), 90);
    }
}
