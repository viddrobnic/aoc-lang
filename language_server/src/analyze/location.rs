use parser::position::{Position, PositionOrdering, Range};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Inserted location isn't after current locations.")]
    InvalidRange,
}

/// A single piece of information for a specific
/// location. Could be where the symbols at `location`
/// is defined, references for symbol at `location` or more.
#[derive(Debug, PartialEq, Eq)]
pub struct LocationEntry<T> {
    pub location: Range,
    pub entry: T,
}

/// Data structure for holding multiple entries of
/// location connected data and enables fast
/// querying.
#[derive(Debug, PartialEq, Eq, Default)]
pub struct LocationData<T>(Vec<LocationEntry<T>>);

impl<T> LocationData<T> {
    pub fn new() -> Self {
        Self(vec![])
    }

    /// Add location entry. The way this data structure is built, entry's
    /// location must be after all other locations in the data strcture.
    pub fn push(&mut self, entry: LocationEntry<T>) -> Result<(), Error> {
        // Check that location is valid
        if let Some(last) = self.0.last() {
            if entry.location.start.cmp_range(&last.location) != PositionOrdering::After {
                return Err(Error::InvalidRange);
            }
        }

        self.0.push(entry);
        Ok(())
    }

    // Get the index of entry in which position is located
    fn bisect(&self, position: &Position) -> Option<usize> {
        let mut start = 0;
        let mut end = self.0.len();

        loop {
            let len = end - start;
            if len == 0 {
                return None;
            }

            if len == 1 {
                return match position.cmp_range(&self.0[start].location) {
                    PositionOrdering::Inside => Some(start),
                    _ => None,
                };
            }

            let middle = start + len / 2;
            match position.cmp_range(&self.0[middle].location) {
                PositionOrdering::Before => end = middle,
                PositionOrdering::Inside => return Some(middle),
                PositionOrdering::After => start = middle,
            }
        }
    }

    /// Get reference to location entry which contains the given position.
    pub fn get(&self, position: &Position) -> Option<&LocationEntry<T>> {
        let idx = self.bisect(position)?;
        Some(&self.0[idx])
    }

    /// Get mutable reference to location entry which contains the given position.
    pub fn get_mut(&mut self, position: &Position) -> Option<&mut LocationEntry<T>> {
        let idx = self.bisect(position)?;
        Some(&mut self.0[idx])
    }
}

#[cfg(test)]
mod test {
    use parser::position::{Position, Range};

    use super::{LocationData, LocationEntry};

    #[test]
    fn bisect() {
        let mut data = LocationData::<()>::new();
        data.push(LocationEntry {
            location: Range {
                start: Position::new(2, 2),
                end: Position::new(2, 10),
            },
            entry: (),
        })
        .unwrap();
        data.push(LocationEntry {
            location: Range {
                start: Position::new(4, 0),
                end: Position::new(5, 0),
            },
            entry: (),
        })
        .unwrap();
        data.push(LocationEntry {
            location: Range {
                start: Position::new(7, 0),
                end: Position::new(9, 3),
            },
            entry: (),
        })
        .unwrap();

        let tests = [
            (Position::new(0, 0), None),
            (Position::new(2, 2), Some(0)),
            (Position::new(2, 9), Some(0)),
            (Position::new(2, 10), None),
            (Position::new(3, 5123), None),
            (Position::new(4, 0), Some(1)),
            (Position::new(4, 42069), Some(1)),
            (Position::new(5, 0), None),
            (Position::new(8, 0), Some(2)),
            (Position::new(8, 123), Some(2)),
            (Position::new(7, 0), Some(2)),
            (Position::new(9, 2), Some(2)),
            (Position::new(9, 3), None),
            (Position::new(10, 31), None),
        ];

        for (pos, idx) in tests {
            assert_eq!(idx, data.bisect(&pos));
        }
    }
}
