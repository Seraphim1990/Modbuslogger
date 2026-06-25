use crate::messages::requests::measure_request::HashedValue;
pub struct ValueHasher {
    hashed_values: Vec<HashedValue>,
    size: usize,
    capacity: usize,
    current: usize,
    // TODO: винести в toml конфіг
    stable_delta: i64,
    failure_delta: i64,
    in_failure: bool,
    failure_reported: bool,
}

impl ValueHasher {
    pub fn new(capacity: usize, val: f64, timestamp: i64) -> ValueHasher {
        let mut hashed_values = vec![HashedValue::default(); capacity];
        hashed_values[0] = HashedValue { val, timestamp };
        ValueHasher {
            hashed_values,
            size: 1,
            current: 0,
            capacity,
            stable_delta: 300,
            failure_delta: 120,
            in_failure: false,
            failure_reported: false,
        }
    }

    pub fn add(&mut self, val: f64, timestamp: i64) -> Result<Option<HashedValue>, HashedValue> {
        if val == f64::MIN {
            if !self.in_failure {
                self.advance_and_write(val, timestamp);
                self.in_failure = true;
                return Ok(None);
            }
            if !self.failure_reported {
                if timestamp - self.hashed_values[self.current].timestamp >= self.failure_delta {
                    self.hashed_values[self.current].timestamp = timestamp;
                    self.failure_reported = true;
                    return Err(self.hashed_values[self.current].clone());
                }
                return Ok(None);
            }
            return Ok(None);
        }

        // val != f64::MIN
        if self.failure_reported {
            self.in_failure = false;
            self.failure_reported = false;
            self.advance_and_write(val, timestamp);
            return Ok(Some(self.hashed_values[self.current].clone()));
        }

        if self.in_failure {
            self.in_failure = false;
            let prev_idx = self.current.checked_sub(1)
                .unwrap_or(self.capacity - 1);
            let prev_val = self.hashed_values[prev_idx].val;
            let prev_ts = self.hashed_values[prev_idx].timestamp;

            if val != prev_val || timestamp - prev_ts >= self.stable_delta {
                self.hashed_values[self.current] = HashedValue { val, timestamp };
                return Ok(Some(self.hashed_values[self.current].clone()));
            }
            // невдача була шумом — відкочуємо каретку
            self.current = prev_idx;
            self.hashed_values[self.current].timestamp = timestamp;
            return Ok(None);
        }

        // нормальний режим
        let prev_val = self.hashed_values[self.current].val;
        let prev_ts = self.hashed_values[self.current].timestamp;
        if val != prev_val || timestamp - prev_ts >= self.stable_delta {
            self.advance_and_write(val, timestamp);
            return Ok(Some(self.hashed_values[self.current].clone()));
        }
        self.hashed_values[self.current].timestamp = timestamp;
        Ok(None)
    }

    pub fn get_hashed(&self, from: i64, to: i64) -> Option<Vec<HashedValue>> {
        if from >= to || self.size == 0 {
            return None;
        }

        let oldest_idx = if self.size < self.capacity {
            0
        } else {
            (self.current + 1) % self.capacity
        };

        if self.hashed_values[oldest_idx].timestamp > from as i64 {
            return None;
        }

        let start = Self::lower_bound_ring(&self.hashed_values, self.size, oldest_idx, from);
        let end = Self::lower_bound_ring(&self.hashed_values, self.size, oldest_idx, to);

        if start == end {
            return None;
        }

        let mut result = Vec::with_capacity(end - start);
        for i in start..end {
            let idx = (oldest_idx + i) % self.capacity;
            result.push(self.hashed_values[idx].clone());
        }

        Some(result)
    }

    fn advance_and_write(&mut self, val: f64, timestamp: i64) {
        let next = if self.size == 0 {
            0
        } else {
            (self.current + 1) % self.capacity
        };
        self.hashed_values[next] = HashedValue { val, timestamp };
        self.current = next;
        if self.size < self.capacity {
            self.size += 1;
        }
    }

    fn lower_bound_ring(
        arr: &[HashedValue],
        size: usize,
        oldest_idx: usize,
        target: i64,
    ) -> usize {
        let len = arr.len();
        let mut left = 0usize;
        let mut right = size;

        while left < right {
            let mid = left + (right - left) / 2;
            let idx = (oldest_idx + mid) % len;

            if arr[idx].timestamp < target {
                left = mid + 1;
            } else {
                right = mid;
            }
        }
        left
    }
}