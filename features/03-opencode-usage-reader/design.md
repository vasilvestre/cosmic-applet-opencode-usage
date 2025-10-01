# Technical Design: OpenCode Usage Reader

## Architecture Overview

The OpenCode Usage Reader module will consist of three main components:
1. **Storage Scanner**: Discovers and reads JSON files from the filesystem
2. **Usage Parser**: Parses individual JSON files into structured data
3. **Aggregator**: Collects and aggregates usage metrics

## Component Design

### 1. Storage Scanner (`scanner.rs`)

**Purpose**: Efficiently discover and read usage files from OpenCode storage

**Key Types**:
```rust
pub struct StorageScanner {
    storage_path: PathBuf,
}

impl StorageScanner {
    pub fn new() -> Result<Self, ScannerError>;
    pub fn scan(&self) -> Result<Vec<PathBuf>, ScannerError>;
    pub fn scan_async(&self) -> impl Stream<Item = Result<PathBuf, ScannerError>>;
}
```

**Algorithm**:
1. Resolve storage path: `~/.local/share/opencode/storage/part/`
2. Use `walkdir` to recursively traverse directory structure
3. Filter files ending with `.json`
4. Return iterator of matching paths

**Performance Considerations**:
- Use `walkdir` with parallelism for large directories
- Implement early termination on errors
- Stream results to avoid loading all paths in memory

### 2. Usage Parser (`parser.rs`)

**Purpose**: Parse individual JSON files into typed Rust structures

**Key Types**:
```rust
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct UsagePart {
    pub id: String,
    #[serde(rename = "messageID")]
    pub message_id: String,
    #[serde(rename = "sessionID")]
    pub session_id: String,
    #[serde(rename = "type")]
    pub event_type: String,
    pub tokens: Option<TokenUsage>,
    pub cost: f64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TokenUsage {
    pub input: u64,
    pub output: u64,
    pub reasoning: u64,
    pub cache: CacheUsage,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CacheUsage {
    pub write: u64,
    pub read: u64,
}

pub struct UsageParser;

impl UsageParser {
    pub fn parse_file(path: &Path) -> Result<Option<UsagePart>, ParserError>;
    pub fn parse_json(content: &str) -> Result<Option<UsagePart>, ParserError>;
}
```

**Parsing Strategy**:
- Use `serde_json` for deserialization
- Return `Option<UsagePart>` to handle files without token data
- Skip files that don't match expected structure (defensive parsing)
- Log warnings for malformed files but don't fail the entire scan

### 3. Aggregator (`aggregator.rs`)

**Purpose**: Aggregate usage metrics from multiple parsed parts

**Key Types**:
```rust
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UsageMetrics {
    pub total_input_tokens: u64,
    pub total_output_tokens: u64,
    pub total_reasoning_tokens: u64,
    pub total_cache_write_tokens: u64,
    pub total_cache_read_tokens: u64,
    pub total_cost: f64,
    pub total_interactions: u64,
    pub last_updated: chrono::DateTime<chrono::Utc>,
}

pub struct UsageAggregator {
    metrics: UsageMetrics,
}

impl UsageAggregator {
    pub fn new() -> Self;
    pub fn add_part(&mut self, part: &UsagePart);
    pub fn finalize(self) -> UsageMetrics;
}
```

**Aggregation Logic**:
- Initialize counters to zero
- For each `UsagePart` with tokens:
  - Add input/output/reasoning tokens to respective totals
  - Add cache read/write to respective totals
  - Accumulate cost
  - Increment interaction count
- Set `last_updated` timestamp when finalized

### 4. Main Module (`opencode.rs`)

**Purpose**: Orchestrate scanning, parsing, and aggregation with caching

**Key Types**:
```rust
pub struct OpenCodeUsageReader {
    scanner: StorageScanner,
    cache: Option<CachedMetrics>,
}

#[derive(Debug, Clone)]
struct CachedMetrics {
    metrics: UsageMetrics,
    cached_at: chrono::DateTime<chrono::Utc>,
}

impl OpenCodeUsageReader {
    pub fn new() -> Result<Self, ReaderError>;
    
    pub async fn get_usage(&mut self) -> Result<UsageMetrics, ReaderError>;
    
    async fn scan_and_aggregate(&self) -> Result<UsageMetrics, ReaderError>;
    
    fn should_refresh_cache(&self) -> bool;
}
```

**Caching Strategy**:
- Cache metrics in memory with timestamp
- Refresh cache if older than 5 minutes
- Invalidate cache on explicit refresh request
- No persistent cache (simple in-memory only)

**Main Algorithm** (`get_usage`):
```
1. Check if cache exists and is fresh (< 5 minutes old)
   - If yes: return cached metrics
   - If no: proceed to step 2
   
2. Scan storage directory for JSON files
3. For each file:
   a. Read file contents
   b. Parse JSON into UsagePart
   c. If successful and has tokens, add to aggregator
   d. If error, log and continue
   
4. Finalize aggregation to get metrics
5. Cache metrics with current timestamp
6. Return metrics
```

## Error Handling

**Error Types**:
```rust
#[derive(Debug, thiserror::Error)]
pub enum ReaderError {
    #[error("Storage directory not found: {0}")]
    StorageNotFound(PathBuf),
    
    #[error("Permission denied accessing storage: {0}")]
    PermissionDenied(PathBuf),
    
    #[error("Failed to scan directory: {0}")]
    ScanError(String),
    
    #[error("No usage data found")]
    NoDataFound,
}

#[derive(Debug, thiserror::Error)]
pub enum ParserError {
    #[error("Failed to read file: {0}")]
    FileReadError(#[from] std::io::Error),
    
    #[error("Invalid JSON: {0}")]
    JsonError(#[from] serde_json::Error),
}
```

**Error Handling Strategy**:
- Scanner errors: Propagate to caller (fatal)
- Parser errors: Log and skip individual files (non-fatal)
- Missing storage: Return clear error to user

## Testing Strategy

### Unit Tests

1. **Parser Tests**:
   - Valid JSON with complete token data
   - Valid JSON with missing optional fields
   - JSON without token data (should return None)
   - Malformed JSON (should return error)
   - Empty file

2. **Aggregator Tests**:
   - Aggregate single part
   - Aggregate multiple parts
   - Aggregate with zero values
   - Verify interaction counting

3. **Scanner Tests** (with temp directory):
   - Empty directory
   - Directory with JSON files
   - Directory with mixed file types
   - Nested directory structure

### Integration Tests

1. Create temporary directory structure mimicking OpenCode storage
2. Populate with sample JSON files
3. Run full scan and aggregation
4. Verify correct metrics

## Performance Considerations

- **File Count**: Assume 10k-100k files for typical usage
- **Memory**: Process files in streaming fashion, don't load all in memory
- **Time Budget**: Target < 5 seconds for full scan
- **Parallelism**: Use `rayon` or async tasks for parallel file processing

## Dependencies

```toml
walkdir = "2.4"          # Directory traversal
serde_json = "1.0"       # JSON parsing
tokio = { features = ["fs"] }  # Async file I/O
chrono = "0.4"           # Timestamps
thiserror = "2.0"        # Error handling
```

## Integration Points

- **Configuration Module**: Get storage path (default to standard location)
- **UI Module**: Provide metrics for display
- **App State**: Store `OpenCodeUsageReader` instance

## Future Enhancements

1. **Persistent Cache**: Save cache to disk to avoid rescan on restart
2. **Incremental Scanning**: Track last scanned timestamp, only scan newer files
3. **File Watching**: Use `notify` crate to watch for new files
4. **Filtering**: Add date range and session ID filters
5. **Export**: Generate JSONL export for compatibility with `ccusage` tool
