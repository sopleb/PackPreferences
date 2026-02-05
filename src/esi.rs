use anyhow::{Context, Result};
use serde::Deserialize;
use std::collections::HashMap;

const ESI_NAMES_ENDPOINT: &str = "https://esi.evetech.net/latest/universe/names/";
const BATCH_LIMIT: usize = 500;

#[derive(Debug, Deserialize)]
struct EsiNameResult {
    id: u64,
    name: String,
    #[allow(dead_code)]
    category: String,
}

/// Resolves character IDs to names via ESI API.
pub fn resolve_character_names(character_ids: &[u64]) -> Result<HashMap<u64, String>> {
    let mut results = HashMap::new();

    if character_ids.is_empty() {
        return Ok(results);
    }

    // Process in batches of 500
    for chunk in character_ids.chunks(BATCH_LIMIT) {
        let batch_results = fetch_names_batch(chunk)?;
        results.extend(batch_results);
    }

    Ok(results)
}

fn fetch_names_batch(ids: &[u64]) -> Result<HashMap<u64, String>> {
    let client = reqwest::blocking::Client::new();

    let response = client
        .post(ESI_NAMES_ENDPOINT)
        .json(&ids)
        .send()
        .context("Failed to send ESI request")?;

    if !response.status().is_success() {
        // Some IDs might not exist, that's okay
        if response.status().as_u16() == 404 {
            return Ok(HashMap::new());
        }
        anyhow::bail!("ESI request failed with status: {}", response.status());
    }

    let names: Vec<EsiNameResult> = response.json().context("Failed to parse ESI response")?;

    Ok(names
        .into_iter()
        .filter(|n| n.category == "character")
        .map(|n| (n.id, n.name))
        .collect())
}

/// Resolves character names with caching support.
/// Returns updated cache entries.
pub fn resolve_with_cache(
    character_ids: &[u64],
    cache: &HashMap<u64, String>,
) -> Result<HashMap<u64, String>> {
    // Find IDs not in cache
    let uncached: Vec<u64> = character_ids
        .iter()
        .filter(|id| !cache.contains_key(id))
        .copied()
        .collect();

    if uncached.is_empty() {
        return Ok(HashMap::new());
    }

    resolve_character_names(&uncached)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_ids() {
        let result = resolve_character_names(&[]).unwrap();
        assert!(result.is_empty());
    }
}
