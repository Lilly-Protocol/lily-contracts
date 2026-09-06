# Payment intent indexes

The payments contract stores each full intent under its numeric ID and maintains a
payer-specific persistent vector of intent IDs. The index preserves creation order
and powers cursor-based reads through `list_intents(payer, cursor, limit)`.

## Pagination

- `cursor` is the zero-based position in that payer's intent history.
- `limit` must be between 1 and `MAX_INTENTS_PAGE_SIZE` (100), inclusive.
- Results are returned in creation order.
- A cursor at or beyond the end returns an empty vector.

## Storage and execution cost

Each new intent adds one `u64` to the payer's index in addition to the existing full
intent record. Because the vector is stored as one value, appending rewrites that
payer's index and its write cost grows with the payer's history. Reads hydrate at
most 100 full intent records, so query work is bounded by the requested page size.
If payer histories become large, the index should migrate to chunked pages without
changing the public cursor semantics.
