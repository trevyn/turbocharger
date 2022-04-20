## Usage

Your API is defined declaratively by functions annotated with `#[backend]`:

```rust,ignore
use turbocharger::{prelude::*, backend};

#[backend]
pub async fn get_one() -> i64 {
 1
}
```
