//! Checks on the `mt-draft inspect --json` document used by the opt-in tests.

use serde_json::Value;

/// The reported players, once their `slot` values are exactly `0..active`, in order.
///
/// Counting players is not enough: a report can keep the count while omitting some
/// slots and repeating others (issue #8). Anything else is an error naming the slots.
pub fn checked_players(doc: &Value, active: usize) -> Result<&Vec<Value>, String> {
    let players = doc["players"]
        .as_array()
        .ok_or_else(|| "inspect --json has no players array".to_string())?;
    let reported: Vec<Option<u64>> = players.iter().map(|p| p["slot"].as_u64()).collect();
    let expected: Vec<Option<u64>> = (0..active as u64).map(Some).collect();
    if reported == expected {
        return Ok(players);
    }
    let first_wrong = reported
        .iter()
        .zip(&expected)
        .position(|(r, e)| r != e)
        .unwrap_or(reported.len().min(expected.len()));
    let mut seen = vec![0usize; active];
    let mut foreign = Vec::new();
    for r in &reported {
        match r {
            Some(s) if (*s as usize) < active => seen[*s as usize] += 1,
            other => foreign.push(*other),
        }
    }
    let missing: Vec<usize> = (0..active).filter(|&s| seen[s] == 0).collect();
    let repeated: Vec<usize> = (0..active).filter(|&s| seen[s] > 1).collect();
    Err(format!(
        "inspect --json reported {} players for {active} active slots, not slots 0..{active} \
         in order: first difference at position {first_wrong}; missing {} {:?}; repeated {:?}; \
         outside the active range or not a number {:?}",
        reported.len(),
        missing.len(),
        &missing[..missing.len().min(10)],
        &repeated[..repeated.len().min(10)],
        &foreign[..foreign.len().min(10)],
    ))
}
