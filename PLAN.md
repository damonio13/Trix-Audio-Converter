# Plan: Fix Remaining Rust Duplications

## Fix #8: album_art.rs — Extract `fetch_json` helper

**Current duplication:** `search_musicbrainz`, `search_deezer`, `search_itunes` all repeat HTTP GET + JSON parse.

**Approach:** Extract a private `fetch_json` method that handles HTTP GET with optional headers + JSON deserialization. Each search function calls it and keeps its own extraction logic.

```rust
async fn fetch_json(&self, url: &str, headers: &[(&str, &str)]) -> Result<serde_json::Value, String> {
    let mut req = self.client.get(url);
    for (k, v) in headers {
        req = req.header(*k, *v);
    }
    let resp = req.send().await
        .map_err(|e| format!("HTTP request failed: {}", e))?;
    resp.json().await
        .map_err(|e| format!("JSON parse failed: {}", e))
}
```

- **MusicBrainz** (lines 25-71): call `self.fetch_json(&url, &[("User-Agent", "AudioMasterPro/1.0 (contact@audiomaster.pro)")]).await?` then keep its unique HEAD-request verification logic.
- **Deezer** (lines 73-111): call `self.fetch_json(&url, &[]).await?` then keep its `cover_xl`/`cover_big` extraction.
- **iTunes** (lines 113-145): call `self.fetch_json(&url, &[]).await?` then keep its `artworkUrl100` → `600x600` replacement.

**Edits:**
1. Add `fetch_json` method in `impl AlbumArtDownloader` (after `new()`).
2. In `search_musicbrainz`: replace lines 32-39 with `let data = self.fetch_json(&url, &[("User-Agent", "AudioMasterPro/1.0 (contact@audiomaster.pro)")]).await?;`
3. In `search_deezer`: replace lines 80-86 with `let data = self.fetch_json(&url, &[]).await?;`
4. In `search_itunes`: replace lines 120-126 with `let data = self.fetch_json(&url, &[]).await?;`

---

## Fix #9: metadata.rs — Extract `merge_tags` function

**Current duplication:** Lines 41-47 and 48-54 are identical loops merging JSON objects into `all_tags`.

**Approach:** Extract a free function `merge_tags`.

```rust
fn merge_tags(target: &mut HashMap<String, String>, source: &serde_json::Value) {
    if let Some(obj) = source.as_object() {
        for (k, v) in obj {
            if let Some(s) = v.as_str() {
                target.insert(k.clone(), s.to_string());
            }
        }
    }
}
```

**Edits:**
1. Add `merge_tags` function before `impl AudioMetadata`.
2. Replace lines 41-54 with:
   ```rust
   merge_tags(&mut all_tags, &tags);
   merge_tags(&mut all_tags, &stream_tags);
   ```

---

## Fix #10: internet_radio.rs — Extract `validate_stream_params`

**Current duplication:** `start_stream` (lines 52-60) and `start_icecast` (lines 111-128) share 3 validation checks: `is_safe_path`, `is_valid_port`, `is_valid_bitrate`.

**Approach:** Extract common validation into `validate_stream_params`. `start_icecast` calls it first, then does its additional hostname/password/mount checks.

```rust
fn validate_stream_params(input: &str, port: u16, bitrate: u32) -> Result<(), String> {
    if !crate::utils::is_safe_path(input) {
        return Err("Caminho de entrada invalido".into());
    }
    if !is_valid_port(port) {
        return Err(format!("Porta invalida: {}. Use 1024-65535.", port));
    }
    if !is_valid_bitrate(bitrate) {
        return Err(format!("Bitrate invalido: {}kbps. Use 32-512.", bitrate));
    }
    Ok(())
}
```

**Edits:**
1. Add `validate_stream_params` function before `impl InternetRadio`.
2. In `start_stream`: replace lines 52-60 with `validate_stream_params(input, port, bitrate)?;`
3. In `start_icecast`: replace lines 111-128 with:
   ```rust
   validate_stream_params(input, port, bitrate)?;
   if !is_valid_hostname(server) {
       return Err(format!("Servidor invalido: '{}'. Use hostname ou IP valido.", server));
   }
   if !is_valid_password(password) {
       return Err("Senha invalida. Use apenas alfanumericos e !@#$%^&*()_+-=".into());
   }
   if !is_valid_mount(mount) {
       return Err(format!("Mount point invalido: '{}'. Use apenas alfanumericos, /, - ou _.", mount));
   }
   ```

---

## Fix #15: batch_rules.rs — Extract `compare_numeric`

**Current duplication:** `bitrate` (lines 71-83), `sample_rate` (lines 85-97), `channels` (lines 98-110) all parse two `u32` values and match on an operator.

**Approach:** Extract a `compare_numeric` helper.

```rust
fn compare_numeric(val: Option<&str>, target: &str, op: &str) -> bool {
    let value: u32 = val.and_then(|v| v.parse().ok()).unwrap_or(0);
    let target: u32 = target.parse().unwrap_or(0);
    match op {
        "less_than" => value < target,
        "greater_than" => value > target,
        "equals" => value == target,
        "not_equals" => value != target,
        _ => false,
    }
}
```

**Edits:**
1. Add `compare_numeric` function before `impl BatchRulesEngine`.
2. Replace the `"bitrate"` arm (lines 70-83) with:
   ```rust
   "bitrate" => compare_numeric(metadata.get("bitrate"), &rule.condition.value, &rule.condition.operator),
   ```
3. Replace the `"sample_rate"` arm (lines 84-97) with:
   ```rust
   "sample_rate" => compare_numeric(metadata.get("sample_rate"), &rule.condition.value, &rule.condition.operator),
   ```
4. Replace the `"channels"` arm (lines 98-110) with:
   ```rust
   "channels" => compare_numeric(metadata.get("channels"), &rule.condition.value, &rule.condition.operator),
   ```

---

## Verification

After all edits, run `cargo check` in `src-rs/` directory to confirm no compilation errors.
