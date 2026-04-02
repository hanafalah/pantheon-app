# rust-core

Core helper library for Pantheon App - Laravel Support equivalent.

## Features

### String Helpers
- `str_slug()` - Convert to slug format
- `str_snake()` - Convert to snake_case
- `str_camel()` - Convert to camelCase
- `str_pascal()` - Convert to PascalCase
- `str_random()` - Generate random string
- `str_limit()` - Truncate string with suffix
- `str_starts_with_any()` - Check if starts with any needle
- `str_ends_with_any()` - Check if ends with any needle
- `str_contains_any()` - Check if contains any needle

### Collection Helpers
- `chunk()` - Chunk array into smaller arrays
- `pluck()` - Pluck values by key
- `group_by()` - Group items by key function
- `first()` - Get first item matching predicate
- `last()` - Get last item matching predicate
- `every()` - Check if all match predicate
- `some()` - Check if any match predicate
- `unique()` - Get unique items
- `flatten()` - Flatten nested arrays
- `partition()` - Partition by predicate

### Date/Time Helpers
- `now()` - Get current UTC datetime
- `timestamp()` - Get timestamp in seconds
- `timestamp_millis()` - Get timestamp in milliseconds
- `parse_date()` - Parse date from string
- `format_date()` - Format date to string
- `format_datetime()` - Format to ISO 8601
- `add_days()` / `sub_days()` - Add/subtract days
- `add_hours()` / `add_minutes()` - Add hours/minutes
- `diff_days()` / `diff_hours()` - Calculate difference
- `is_past()` / `is_future()` - Check if past/future
- `start_of_day()` / `end_of_day()` - Get day boundaries

### UUID Helpers
- `generate_uuid()` - Generate new UUID v4
- `uuid_from_string()` - Parse UUID from string
- `uuid_to_compact()` - Convert to compact format (no hyphens)
- `uuid_from_compact()` - Parse from compact format
- `is_valid_uuid()` - Validate UUID string
- `generate_uuids()` - Generate multiple UUIDs

### Macros
- `impl_getters!()` - Auto-implement getter methods
- `impl_builder!()` - Auto-implement builder pattern
- `impl_display!()` - Auto-implement Display trait
- `impl_from_error!()` - Auto-implement From for errors
- `new_entity!()` - Create new entity with UUID and timestamps
- `derive_common!()` - Derive common traits

## Usage

```rust
use rust_core::*;

// String helpers
let slug = str_slug("Hello World"); // "hello-world"
let snake = str_snake("HelloWorld"); // "hello_world"

// Collection helpers
let items = vec![1, 2, 3, 4, 5];
let chunks = chunk(&items, 2); // [[1, 2], [3, 4], [5]]

// Date helpers
let now = now();
let future = add_days(&now, 7);

// UUID helpers
let uuid = generate_uuid();
let compact = uuid_to_compact(&uuid);
```

## Testing

All helpers include comprehensive unit tests:

```bash
cargo test -p rust-core
```
